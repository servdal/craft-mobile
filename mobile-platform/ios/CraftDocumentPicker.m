// SPDX-License-Identifier: MIT OR Apache-2.0
#import <UIKit/UIKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#import <PhotosUI/PhotosUI.h>
#import <Photos/Photos.h>
#import <ImageIO/ImageIO.h>
#include <stdlib.h>
#include <string.h>
static NSObject *gate;
static NSString *reply;
static BOOL busy;
static NSUInteger activeToken;
static id activeDelegate;
static void initializeGate(void) { static dispatch_once_t once;dispatch_once(&once, ^{gate=[NSObject new];}); }
static void finish(NSDictionary *result, NSUInteger token) {
 initializeGate();NSData *data=[NSJSONSerialization dataWithJSONObject:result options:0 error:nil];
 @synchronized(gate){if(!busy||token!=activeToken)return;reply=[[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];busy=NO;activeDelegate=nil;}
}
static UIViewController *presenter(void) {
 for(UIScene *scene in UIApplication.sharedApplication.connectedScenes) {
  if(![scene isKindOfClass:UIWindowScene.class]||scene.activationState!=UISceneActivationStateForegroundActive)continue;
  for(UIWindow *window in ((UIWindowScene *)scene).windows)if(window.isKeyWindow){UIViewController *vc=window.rootViewController;while(vc.presentedViewController)vc=vc.presentedViewController;return vc;}
 }
 return nil;
}
@interface CraftDocumentsDelegate:NSObject<UIDocumentPickerDelegate,UIAdaptivePresentationControllerDelegate>
@property(nonatomic,copy)NSString *exportPath;
@property(nonatomic,assign)NSUInteger token;
@end
@implementation CraftDocumentsDelegate
-(void)documentPickerWasCancelled:(UIDocumentPickerViewController *)controller {finish(@{},self.token);}
-(void)presentationControllerDidDismiss:(UIPresentationController *)controller {finish(@{},self.token);}
-(void)documentPicker:(UIDocumentPickerViewController *)controller didPickDocumentsAtURLs:(NSArray<NSURL *> *)urls {
 NSUInteger token=self.token;
 if(self.exportPath){finish(@{@"path":self.exportPath},token);return;}
 NSURL *source=urls.firstObject;if(!source){finish(@{},token);return;}
 dispatch_async(dispatch_get_global_queue(QOS_CLASS_USER_INITIATED,0), ^{ @autoreleasepool {
 BOOL scoped=[source startAccessingSecurityScopedResource];
 NSURL *documents=[NSFileManager.defaultManager URLsForDirectory:NSDocumentDirectory inDomains:NSUserDomainMask].firstObject;
 NSURL *imports=[documents URLByAppendingPathComponent:@"Imports" isDirectory:YES];
 NSString *rootPath=documents.URLByResolvingSymlinksInPath.path;
 NSString *importPath=imports.URLByResolvingSymlinksInPath.path;
 if(!documents||!source.lastPathComponent.length||![importPath hasPrefix:[rootPath stringByAppendingString:@"/"]]) {
  if(scoped)[source stopAccessingSecurityScopedResource];finish(@{@"error":@"The application import folder is unavailable"},token);return;
 }
 NSURL *folder=[imports URLByAppendingPathComponent:NSUUID.UUID.UUIDString isDirectory:YES];
 NSError *failure=nil;
 if(![NSFileManager.defaultManager createDirectoryAtURL:folder withIntermediateDirectories:YES attributes:nil error:&failure]) {
  if(scoped)[source stopAccessingSecurityScopedResource];finish(@{@"error":failure.localizedDescription?:@"Could not create import folder"},token);return;
 }
 NSURL *destination=[folder URLByAppendingPathComponent:source.lastPathComponent];
 NSFileCoordinator *coordinator=[[NSFileCoordinator alloc] initWithFilePresenter:nil];
 __block NSError *copyError=nil;
 [coordinator coordinateReadingItemAtURL:source options:NSFileCoordinatorReadingWithoutChanges error:&failure byAccessor:^(NSURL *url){
  NSNumber *directory=nil;[url getResourceValue:&directory forKey:NSURLIsDirectoryKey error:&copyError];
  if(directory.boolValue){copyError=[NSError errorWithDomain:@"CraftDocuments" code:1 userInfo:@{NSLocalizedDescriptionKey:@"Choose a document, rather than a folder"}];return;}
  [NSFileManager.defaultManager copyItemAtURL:url toURL:destination error:&copyError];
 }];
 if(scoped)[source stopAccessingSecurityScopedResource];
 if(failure||copyError){[NSFileManager.defaultManager removeItemAtURL:folder error:nil];finish(@{@"error":(failure?:copyError).localizedDescription?:@"Import failed"},token);}
 else finish(@{@"path":destination.path},token);
 }});
}
@end
static BOOL begin(NSString *exportPath) {
 initializeGate();NSUInteger token;@synchronized(gate){if(busy||reply)return NO;busy=YES;token=++activeToken;}
 dispatch_async(dispatch_get_main_queue(), ^{
  UIViewController *vc=presenter();if(!vc){finish(@{@"error":@"Files could not be opened. Try again when the app is active."},token);return;}
  CraftDocumentsDelegate *delegate=[CraftDocumentsDelegate new];delegate.exportPath=exportPath;delegate.token=token;activeDelegate=delegate;
  UIDocumentPickerViewController *picker=exportPath?[[UIDocumentPickerViewController alloc] initForExportingURLs:@[[NSURL fileURLWithPath:exportPath]] asCopy:YES]:[[UIDocumentPickerViewController alloc] initForOpeningContentTypes:@[UTTypeItem] asCopy:YES];
  picker.delegate=delegate;picker.allowsMultipleSelection=NO;picker.shouldShowFileExtensions=YES;
  picker.modalPresentationStyle=UIModalPresentationFormSheet;
  [vc presentViewController:picker animated:YES completion:nil];picker.presentationController.delegate=delegate;
 });return YES;
}
bool craft_documents_open(void){return begin(nil);}
bool craft_documents_export(const char *path){NSString *value=path?[NSString stringWithUTF8String:path]:nil;return value?begin(value):NO;}
char *craft_documents_poll(void){initializeGate();@synchronized(gate){if(!reply)return NULL;char *value=strdup(reply.UTF8String);if(value)reply=nil;return value;}}
void craft_documents_free(char *value){free(value);}

// Photos selections are explicit, single-image imports; no whole-library read permission.
static NSURL *photoImportFolder(NSError **error) {
 NSURL *documents=[NSFileManager.defaultManager URLsForDirectory:NSDocumentDirectory inDomains:NSUserDomainMask].firstObject;
 NSURL *imports=[documents URLByAppendingPathComponent:@"Imports" isDirectory:YES];
 if(!documents||![imports.URLByResolvingSymlinksInPath.path hasPrefix:[documents.URLByResolvingSymlinksInPath.path stringByAppendingString:@"/"]])return nil;
 NSURL *folder=[imports URLByAppendingPathComponent:NSUUID.UUID.UUIDString isDirectory:YES];
 return [NSFileManager.defaultManager createDirectoryAtURL:folder withIntermediateDirectories:YES attributes:nil error:error]?folder:nil;
}
@interface CraftPhotosDelegate:NSObject<PHPickerViewControllerDelegate,UIAdaptivePresentationControllerDelegate>
@property(nonatomic,assign)NSUInteger token;
@end
@implementation CraftPhotosDelegate
-(void)presentationControllerDidDismiss:(UIPresentationController *)controller {finish(@{},self.token);}
-(void)picker:(PHPickerViewController *)picker didFinishPicking:(NSArray<PHPickerResult *> *)results {
 NSUInteger token=self.token;
 [picker dismissViewControllerAnimated:YES completion:nil];
 NSItemProvider *provider=results.firstObject.itemProvider;
 if(!provider){finish(@{},token);return;}
 [provider loadFileRepresentationForTypeIdentifier:UTTypeImage.identifier completionHandler:^(NSURL *url,NSError *loadError){ @autoreleasepool {
  if(!url||loadError){finish(@{@"error":loadError.localizedDescription?:@"Photo could not be loaded"},token);return;}
  CGImageSourceRef source=CGImageSourceCreateWithURL((__bridge CFURLRef)url,NULL);
  NSDictionary *options=@{(id)kCGImageSourceCreateThumbnailFromImageAlways:@YES,(id)kCGImageSourceCreateThumbnailWithTransform:@YES,(id)kCGImageSourceThumbnailMaxPixelSize:@8192};
  CGImageRef decoded=source?CGImageSourceCreateThumbnailAtIndex(source,0,(__bridge CFDictionaryRef)options):NULL;
  if(source)CFRelease(source);
  NSData *png=decoded?UIImagePNGRepresentation([UIImage imageWithCGImage:decoded]):nil;
  if(decoded)CGImageRelease(decoded);
  if(!png||png.length>64*1024*1024){finish(@{@"error":@"Choose a photo that can be imported as a PNG smaller than 64 MB"},token);return;}
  NSError *error=nil;NSURL *folder=photoImportFolder(&error);
  NSURL *destination=[folder URLByAppendingPathComponent:@"Photo.png"];
  if(!folder||![png writeToURL:destination options:NSDataWritingAtomic error:&error]){
   if(folder)[NSFileManager.defaultManager removeItemAtURL:folder error:nil];
   finish(@{@"error":error.localizedDescription?:@"Photo import failed"},token);return;
  }
  finish(@{@"path":destination.path},token);
 }}];
}
@end
bool craft_photos_open(void) {
 initializeGate();NSUInteger token;@synchronized(gate){if(busy||reply)return NO;busy=YES;token=++activeToken;}
 dispatch_async(dispatch_get_main_queue(), ^{
  UIViewController *vc=presenter();if(!vc){finish(@{@"error":@"Photos could not be opened. Try again when the app is active."},token);return;}
  PHPickerConfiguration *config=[[PHPickerConfiguration alloc] init];
  config.filter=PHPickerFilter.imagesFilter;config.selectionLimit=1;
  config.preferredAssetRepresentationMode=PHPickerConfigurationAssetRepresentationModeCompatible;
  CraftPhotosDelegate *delegate=[CraftPhotosDelegate new];delegate.token=token;activeDelegate=delegate;
  PHPickerViewController *picker=[[PHPickerViewController alloc] initWithConfiguration:config];picker.delegate=delegate;
  [vc presentViewController:picker animated:YES completion:nil];picker.presentationController.delegate=delegate;
 });return YES;
}
bool craft_photos_save(const char *path) {
 NSString *value=path?[NSString stringWithUTF8String:path]:nil;if(!value)return NO;
 initializeGate();NSUInteger token;@synchronized(gate){if(busy||reply)return NO;busy=YES;token=++activeToken;}
 NSString *ext=value.pathExtension.lowercaseString;
 if(![@[@"png",@"jpg",@"jpeg",@"heic",@"heif"] containsObject:ext]){finish(@{@"error":@"Export a PNG or JPEG image first, then choose Save to Photos"},token);return YES;}
 CGImageSourceRef source=CGImageSourceCreateWithURL((__bridge CFURLRef)[NSURL fileURLWithPath:value],NULL);
 BOOL valid=source&&CGImageSourceGetCount(source)>0;if(source)CFRelease(source);
 if(!valid){finish(@{@"error":@"The selected file is not a readable image"},token);return YES;}
 void (^save)(PHAuthorizationStatus)=^(PHAuthorizationStatus status){
  if(status!=PHAuthorizationStatusAuthorized&&status!=PHAuthorizationStatusLimited){finish(@{@"error":@"Photos saving is not allowed. You can enable Add Photos access in iOS Settings."},token);return;}
  [PHPhotoLibrary.sharedPhotoLibrary performChanges:^{
   [PHAssetChangeRequest creationRequestForAssetFromImageAtFileURL:[NSURL fileURLWithPath:value]];
  } completionHandler:^(BOOL success,NSError *error){
   finish(success?@{@"path":value}:@{@"error":error.localizedDescription?:@"The image could not be saved to Photos"},token);
  }];
 };
 PHAuthorizationStatus status=[PHPhotoLibrary authorizationStatusForAccessLevel:PHAccessLevelAddOnly];
 if(status==PHAuthorizationStatusNotDetermined)[PHPhotoLibrary requestAuthorizationForAccessLevel:PHAccessLevelAddOnly handler:save];else save(status);
 return YES;
}
