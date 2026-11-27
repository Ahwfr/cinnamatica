#include <stdbool.h>
#import <UIKit/UIKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

typedef void (*PickerCompletion)(void *, const char *);
@interface CinnamaticaDocumentPicker : NSObject <UIDocumentPickerDelegate>
@property(nonatomic) void *context;
@property(nonatomic) PickerCompletion completion;
@property(nonatomic) uint64_t maxBytes;
@property(nonatomic) BOOL finished;
@property(nonatomic,strong) UIDocumentPickerViewController *controller;
- (void)finish:(NSDictionary *)result;
@end
static CinnamaticaDocumentPicker *pending;

@implementation CinnamaticaDocumentPicker
- (void)finish:(NSDictionary *)result {
    if (self.finished) return;
    self.finished=YES;
    NSData *encoded=[NSJSONSerialization dataWithJSONObject:result options:0 error:nil];
    NSString *json=[[NSString alloc] initWithData:encoded encoding:NSUTF8StringEncoding];
    self.completion(self.context,json.UTF8String);
    self.controller.delegate=nil;
    pending=nil;
}
- (void)documentPickerWasCancelled:(UIDocumentPickerViewController *)controller {
    [self finish:@{@"cancelled":@YES}];
}
- (void)documentPicker:(UIDocumentPickerViewController *)controller didPickDocumentsAtURLs:(NSArray<NSURL *> *)urls {
    NSArray *chosen=[urls copy];
    dispatch_async(dispatch_get_global_queue(QOS_CLASS_UTILITY,0), ^{
        NSMutableArray *paths=[NSMutableArray array];
        NSError *failure=nil;
        NSURL *selection=[[NSURL fileURLWithPath:NSTemporaryDirectory() isDirectory:YES]
            URLByAppendingPathComponent:[@"cinnamatica-picks/" stringByAppendingString:NSUUID.UUID.UUIDString] isDirectory:YES];
        if (chosen.count>16) failure=[NSError errorWithDomain:@"Cinnamatica" code:1 userInfo:@{NSLocalizedDescriptionKey:@"Select at most 16 files"}];
        uint64_t total=0;
        NSUInteger index=0;
        for (NSURL *source in chosen) {
            if (failure) break;
            BOOL scoped=[source startAccessingSecurityScopedResource];
            NSFileCoordinator *coordinator=[[NSFileCoordinator alloc] initWithFilePresenter:nil];
            __block NSError *copyError=nil;
            [coordinator coordinateReadingItemAtURL:source options:0 error:&failure byAccessor:^(NSURL *readURL) {
                NSString *name=readURL.lastPathComponent;
                if (!name.length || [name isEqualToString:@"."] || [name isEqualToString:@".."]) {
                    copyError=[NSError errorWithDomain:@"Cinnamatica" code:2 userInfo:@{NSLocalizedDescriptionKey:@"Invalid document name"}]; return;
                }
                NSURL *folder=[selection URLByAppendingPathComponent:[NSString stringWithFormat:@"%lu",(unsigned long)index] isDirectory:YES];
                if (![[NSFileManager defaultManager] createDirectoryAtURL:folder withIntermediateDirectories:YES attributes:nil error:&copyError]) return;
                NSURL *destination=[folder URLByAppendingPathComponent:name isDirectory:NO];
                NSInputStream *input=[NSInputStream inputStreamWithURL:readURL];
                NSOutputStream *output=[NSOutputStream outputStreamWithURL:destination append:NO];
                [input open]; [output open];
                uint8_t buffer[64*1024];
                uint64_t count=0;
                for (;;) {
                    NSInteger got=[input read:buffer maxLength:sizeof(buffer)];
                    if (got==0) break;
                    if (got<0) {copyError=input.streamError ?: [NSError errorWithDomain:@"Cinnamatica" code:5 userInfo:@{NSLocalizedDescriptionKey:@"Could not read document"}];break;}
                    count+=(uint64_t)got;
                    if (count>self.maxBytes || count>256*1024*1024-total) {
                        copyError=[NSError errorWithDomain:@"Cinnamatica" code:3 userInfo:@{NSLocalizedDescriptionKey:@"Selection exceeds the import size limit"}];break;
                    }
                    NSInteger written=0;
                    while (written<got) {
                        NSInteger part=[output write:buffer+written maxLength:(NSUInteger)(got-written)];
                        if (part<=0) {copyError=output.streamError ?: [NSError errorWithDomain:@"Cinnamatica" code:4 userInfo:@{NSLocalizedDescriptionKey:@"Could not copy document"}];break;}
                        written+=part;
                    }
                    if (copyError) break;
                }
                [input close];[output close];
                if (!copyError) [paths addObject:destination.path];
            }];
            if (scoped) [source stopAccessingSecurityScopedResource];
            if (copyError) failure=copyError;
            if (!failure) {
                NSDictionary *attributes=[[NSFileManager defaultManager] attributesOfItemAtPath:paths.lastObject error:&failure];
                total+=[attributes[NSFileSize] unsignedLongLongValue];
            }
            index++;
        }
        if (failure) [[NSFileManager defaultManager] removeItemAtURL:selection error:nil];
        NSDictionary *result=failure?@{@"error":failure.localizedDescription}:@{@"paths":paths};
        dispatch_async(dispatch_get_main_queue(), ^{[self finish:result];});
    });
}
@end

void CinnamaticaPickDocuments(uint64_t maxBytes, bool multiple, void *context, PickerCompletion completion) {
    dispatch_async(dispatch_get_main_queue(), ^{
        if (pending) {completion(context,"{\"error\":\"A document picker is already open\"}");return;}
        UIViewController *presenter=nil;
        for (UIScene *scene in UIApplication.sharedApplication.connectedScenes) {
            if (![scene isKindOfClass:UIWindowScene.class] || scene.activationState!=UISceneActivationStateForegroundActive) continue;
            for (UIWindow *window in ((UIWindowScene *)scene).windows) {
                if (window.isKeyWindow) {presenter=window.rootViewController;break;}
            }
            if (presenter) break;
        }
        while (presenter.presentedViewController) presenter=presenter.presentedViewController;
        if (!presenter) {completion(context,"{\"error\":\"No active iOS window for file selection\"}");return;}
        pending=[CinnamaticaDocumentPicker new];
        pending.context=context;pending.completion=completion;pending.maxBytes=maxBytes;
        pending.controller=[[UIDocumentPickerViewController alloc] initForOpeningContentTypes:@[UTTypeItem] asCopy:YES];
        pending.controller.allowsMultipleSelection=multiple;
        pending.controller.delegate=pending;
        [presenter presentViewController:pending.controller animated:YES completion:nil];
    });
}
