(function($window){


    $window.BotloaderCore = {
        dispatchEvent: () => {},
        dispatchWrapper: async (evt) => {
            $window.BotloaderCore.dispatchEvent(evt);
        },
    }
})(this);