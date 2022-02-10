(function($window){


    $window.BotloaderCore = {
        pendingHandlers: 0,
        dispatchEvent: () => {},
        dispatchWrapper: async (evt) => {
            $window.BotloaderCore.pendingHandlers++;
            $window.BotloaderCore.dispatchEvent(evt).finally(() => {
                $window.BotloaderCore.pendingHandlers--;
                Deno.core.opSync("op_botloader_sync_pending_handlers", $window.BotloaderCore.pendingHandlers);
            })
        },
        trackPromise(promise){
            $window.BotloaderCore.pendingHandlers++;
            Deno.core.opSync("op_botloader_sync_pending_handlers", $window.BotloaderCore.pendingHandlers);
            
            promise.finally(() => {
                $window.BotloaderCore.pendingHandlers--;
                Deno.core.opSync("op_botloader_sync_pending_handlers", $window.BotloaderCore.pendingHandlers);
            })
        }
    }
    // $window.$jackGlobal = {}
    // $window.$jackGlobal = {}

    // $window.$jackGlobal.runEventLoop = async function(cb){
    //     while(true){
    //         const next = await Deno.core.opAsync("op_botloader_rcv_event");
    //         if (next.name === "STOP"){
    //             return;
    //         }
    //         cb(next);
    //     }
    // }
})(this);