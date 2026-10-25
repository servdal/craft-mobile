// SPDX-License-Identifier: MIT OR Apache-2.0
#import <UIKit/UIKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#include <stdlib.h>
#include <string.h>
static NSObject *gate;
static NSString *reply;
static BOOL busy;
static id activeDelegate;
static void initializeGate(void) { static dispatch_once_t once;dispatch_once(&once, ^{gate=[NSObject new];}); }
static void finish(NSDictionary *result) {
 initializeGate();NSData *data=[NSJSONSerialization dataWithJSONObject:result options:0 error:nil];
 @synchronized(gate){if(!busy)return;reply=[[NSString alloc] initWithData:data encoding:NSUTF8StringEncoding];busy=NO;activeDelegate=nil;}
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
@end
@implementation CraftDocumentsDelegate
-(void)documentPickerWasCancelled:(UIDocumentPickerViewController *)controller {finish(@{});}
-(void)presentationControllerDidDismiss:(UIPresentationController *)controller {finish(@{});}
-(void)documentPicker:(UIDocumentPickerViewController *)controller didPickDocumentsAtURLs:(NSArray<NSURL *> *)urls {
 if(self.exportPath){finish(@{@"path":self.exportPath});return;}
 NSURL *source=urls.firstObject;if(!source){finish(@{});return;}
 BOOL scoped=[source startAccessingSecurityScopedResource];
 NSURL *documents=[NSFileManager.defaultManager URLsForDirectory:NSDocumentDirectory inDomains:NSUserDomainMask].firstObject;
 NSURL *folder=[[documents URLByAppendingPathComponent:@"Imports" isDirectory:YES] URLByAppendingPathComponent:NSUUID.UUID.UUIDString isDirectory:YES];
 NSError *failure=nil;
 if(![NSFileManager.defaultManager createDirectoryAtURL:folder withIntermediateDirectories:YES attributes:nil error:&failure]) {
  if(scoped)[source stopAccessingSecurityScopedResource];finish(@{@"error":failure.localizedDescription?:@"Could not create import folder"});return;
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
 if(failure||copyError){[NSFileManager.defaultManager removeItemAtURL:folder error:nil];finish(@{@"error":(failure?:copyError).localizedDescription?:@"Import failed"});}
 else finish(@{@"path":destination.path});
}
@end
static BOOL begin(NSString *exportPath) {
 initializeGate();@synchronized(gate){if(busy||reply)return NO;busy=YES;}
 dispatch_async(dispatch_get_main_queue(), ^{
  UIViewController *vc=presenter();if(!vc){finish(@{@"error":@"Files could not be opened. Try again when the app is active."});return;}
  CraftDocumentsDelegate *delegate=[CraftDocumentsDelegate new];delegate.exportPath=exportPath;activeDelegate=delegate;
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
