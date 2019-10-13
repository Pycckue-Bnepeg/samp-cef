var samp_cef;

if (!samp_cef)
    samp_cef = {};

(function() {
    samp_cef.emit = (event) => {
        native function cef_emit(event);
        return cef_emit(event);
    };

    samp_cef.on = (event) => {
        native function cef_on(event);
        return cef_on(event);
    };

    samp_cef.show_cursor = (show) => {
        native function cef_show_cursor(show);
        return cef_show_cursor(show);
    };
})();
