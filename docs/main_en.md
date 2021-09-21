
## File structure
- The `cef.asi` should be in the game root folder (builded as `loader.dll`).
- Also `cef` folder should be placed there.
- There is a `CEF` dir at `Documents/GTA San Andreas User Files/CEF/`, where is Chromium cookies, caches and so on.
- `gta_sa.exe`
- `cef.asi`
- `cef/`
    - `client.dll`
    - `libcef.dll`
    - `renderer.exe`
    - etc …


## Tips and some limitations
- You should have one browser for all your interfaces to achieve best performance. They can communicate using built-in event system.
- If there is plugins that use relative paths, it could lead to some unexpected things (like `cleo_text` and `cleo_saves` may be placed at `cef` folder). So, please, use absolute paths!

## Pawn API

`cef_create_browser(player_id, browser_id, const url[], hidden, focused)`

Creates a browser for a player. `browser_id` can be any ID (like in SAMP dialogs). `focused` means it hooks all input and passes it in the browser.

`cef_destroy_browser(player_id, browser_id)`

Deletes a browser.

`cef_hide_browser(player_id, browser_id, hide)`

Hides a browser.

`cef_emit_event(player_id, const event_name[], args…)`

Call a client event. Supported types of arguments: `string`, `integer`, `float`.

`cef_subscribe(const event_name[], const callback[])`

Subscribe for client events. Callback signature: `Callback(player_id, const arguments[])`, `arguments` is a string, delimiter of arguments is a space :DDDDD

`cef_player_has_plugin(player_id)`

Check if a player has the plugin.

`cef_create_ext_browser(player_id, browser_id, const texture[], const url[], scale)`

Creates a browser that will be shown on a texture of an object in the future. `scale` arg multiplies texture by this value. `250x30` will be `1250x150` if scale is 5.

`cef_append_to_object(player_id, browser_id, object_id)`

Changes a texture on an object with browser one. The browser should be created with `cef_create_ext_browser`.

`cef_remove_from_object(player_id, browser_id, object_id)`

Return the default texture for an object.

`cef_toggle_dev_tools(player_id, browser_id, enabled)`

Toggles dev tools.

`native cef_set_audio_settings(player_id, browser_id, Float:max_distance, Float:reference_distance)`

Changes audio settings of a browser. `reference_distance` - distance while volume will be 1.0. Then decreases till `max_distance` when it will be 0.

`cef_focus_browser(player_id, browser_id, focused)`

Makes a browser focused as it would be on creation with `focused = true`.

`cef_always_listen_keys(player_id, browser_id, listen)`

The browser starts listen to player input even if it is not focused. This allows you to add a JS key handler on background (`window.addEventListener("keyup")` for example).

`cef_load_url(player_id, browser_id, const url[])`

Loads a new page with a given url (faster than destroy and recreate browser).
### Handlers:

`forward OnCefBrowserCreated(player_id, browser_id, status_code)`

Called when a player has created a browser (from a server or a plugin). `status_code` will be 0 if there is some error. otherwise it is a HTTP response code (200, 404, etc).

`forward OnCefInitialize(player_id, success)`

Called when a player connected to the server with a plugin (or timed-out if there is no installed plugin). Kind of automatic `cef_player_has_plugin`.

## Browser API

`cef.set_focus(focused)`

Set a focus on a browser. It will be rendered last (means have high Z coord). Also can receive mouse and keyboard events.

`cef.on(event_name, callback)`
Subscribes for events from a server / client plugins.

`cef.off(event_name, callback)`

Unsub from an event.

`cef.hide(hide)`

Hides a browser and mutes it.

`cef.emit(event_name, args…)`

Triggers an event with a given name. Arguments can be anything. BUT! On the server it will be a string splited by spaces. In client plugins there is full functionality.

## C API

THIS IS DEPRECATED AND NOT WORKING AT ALL!

Check an rust example using C API (`cef-interface` crate) and `client/external.rs` to check the API.

```C++
    #include <cstdint>
    
    // Do not call next event handlers for this event.
    static const int EXTERNAL_BREAK = 1;
    // Continue handling. If all handlers returns this, server will got the event.
    static const int EXTERNAL_CONTINUE = 0;
    
    using BrowserReadyCallback = void(*)(uint32_t);
    using EventCallback = int(*)(const char*, cef_list_value_t*);
    
    extern "C" {
        // Check if a browser exists.
        bool cef_browser_exists(uint32_t browser);
        // Is a browser ready (created and the page is loaded)
        bool cef_browser_ready(uint32_t browser);
        // Make a request to create a browser.
        void cef_create_browser(uint32_t id, const char *url, bool hidden, bool focused);
        // Create `CefListValue`. THE CLIENT OWNS IT!!!
        cef_list_value_t *cef_create_list();
        // Destroy a browser.
        void cef_destroy_browser(uint32_t id);
        // Trigger an event with given args.
        void cef_emit_event(const char *event, cef_list_value_t *list);
        // Focus a browser.
        void cef_focus_browser(uint32_t id, bool focus);
        // Check if a GTA window is active.
        bool cef_gta_window_active();
        // Hide a browser.
        void cef_hide_browser(uint32_t id, bool hide);
        // Can a browser receive input events now.
        bool cef_input_available(uint32_t browser);
        // Subscribe on browser ready events (like pawn one).
        void cef_on_browser_ready(uint32_t browser, BrowserReadyCallback callback);
        // Kind of deprecated
        bool cef_ready();
        // Subscribe on an event.
        void cef_subscribe(const char *event, EventCallback callback);
        // `cef_input_available` + `cef_focus_browser`, but atomic. This function should be used in this cases.
        bool cef_try_focus_browser(uint32_t browser);
    }
```

Example?: https://gist.github.com/ZOTTCE/5c5bf3b63b1fec29c104e0085cd51f9f
Alseo example? https://gist.github.com/ZOTTCE/7dee2d196138457772aa79355069014a
