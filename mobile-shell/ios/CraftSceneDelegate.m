// Original Craft Mobile lifecycle adapter. SPDX-License-Identifier: MIT OR Apache-2.0
// Keep winit's own window/view; attach it whether created before or after scene connection.
#import <UIKit/UIKit.h>
@interface CraftSceneDelegate : UIResponder <UIWindowSceneDelegate>
@property(nonatomic, strong) UIWindow *window;
@property(nonatomic, weak) UIWindowScene *craftScene;
@end
@implementation CraftSceneDelegate
- (void)attachWindow:(UIWindow *)window {
    if (!self.craftScene || ![NSStringFromClass(window.class) isEqualToString:@"WinitUIWindow"]) return;
    self.window = window;
    if (window.windowScene != self.craftScene) {
        window.windowScene = self.craftScene;
        window.frame = self.craftScene.coordinateSpace.bounds;
        [window makeKeyAndVisible];
        // Refresh winit resize input after scene attachment so egui receives actual safe insets.
        [window.rootViewController.view setNeedsLayout];
        [window.rootViewController.view layoutIfNeeded];
    }
}
- (void)attachExistingWindows {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
    for (UIWindow *window in UIApplication.sharedApplication.windows) [self attachWindow:window];
#pragma clang diagnostic pop
}
- (void)windowBecameVisible:(NSNotification *)notification {
    if (![notification.object isKindOfClass:[UIWindow class]]) return;
    UIWindow *window = notification.object;
    __weak CraftSceneDelegate *weakSelf = self;
    dispatch_async(dispatch_get_main_queue(), ^{ [weakSelf attachWindow:window]; });
}
- (void)scene:(UIScene *)scene willConnectToSession:(UISceneSession *)session options:(UISceneConnectionOptions *)options {
    if (![scene isKindOfClass:[UIWindowScene class]]) return;
    self.craftScene = (UIWindowScene *)scene;
    [NSNotificationCenter.defaultCenter addObserver:self selector:@selector(windowBecameVisible:) name:UIWindowDidBecomeVisibleNotification object:nil];
    [self attachExistingWindows];
    __weak CraftSceneDelegate *weakSelf = self;
    dispatch_async(dispatch_get_main_queue(), ^{ [weakSelf attachExistingWindows]; });
}
- (void)sceneDidDisconnect:(UIScene *)scene {
    [NSNotificationCenter.defaultCenter removeObserver:self name:UIWindowDidBecomeVisibleNotification object:nil];
    self.craftScene = nil;
}
@end
