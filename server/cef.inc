#if !defined CEFSAMP_INC

#define CEFSAMP_INC
enum CEF_Value {
	CEF_STRING = 0,
	CEF_INTEGER = 1,
	CEF_FLOAT = 2
}

#define CEFSTR(%0) CEF_STRING, %0
#define CEFINT(%0) CEF_INTEGER, %0
#define CEFFLOAT(%0) CEF_FLOAT, %0

#define CEF_DEFAULT_MAX_DIST 50.0
#define CEF_DEFAULT_REF_DIST 15.0

native cef_create_browser(player_id, browser_id, const url[], hidden, focused);
native cef_destroy_browser(player_id, browser_id);
native cef_on_player_connect(player_id, const ip[]);
native cef_on_player_disconnect(player_id);
native cef_emit_event(player_id, const event[], {CEF_Value, Float, _}:...);
native cef_subscribe(const event[], const callback[]);
native cef_hide_browser(player_id, browser_id, hide);
native cef_create_ext_browser(player_id, browser_id, const texture[], const url[], scale);
native cef_append_to_object(player_id, browser_id, object_id);
native cef_remove_from_object(player_id, browser_id, object_id);
native cef_player_has_plugin(player_id);
native cef_toggle_dev_tools(player_id, browser_id, enabled);
native cef_set_audio_settings(player_id, browser_id, Float:max_distance, Float:reference_distance);
native cef_focus_browser(player_id, browser_id, focused);
native cef_always_listen_keys(player_id, browser_id, listen);

// internal callbacks hooks

// Connect
public OnPlayerConnect(playerid)
{
    #if defined CEF_OnPlayerConnect
        CEF_OnPlayerConnect(playerid);
    #endif

    new ip_port[22];
    GetPlayerIp(playerid, ip_port, sizeof(ip_port));
    cef_on_player_connect(playerid, ip_port);

    return 1;

}
#if defined _ALS_OnPlayerConnect
    #undef OnPlayerConnect
#else
    #define _ALS_OnPlayerConnect
#endif

#define OnPlayerConnect CEF_OnPlayerConnect

#if defined CEF_OnPlayerConnect
    forward CEF_OnPlayerConnect(playerid);
#endif

// Disconnect
public OnPlayerDisconnect(playerid, reason)
{
    #if defined CEF_OnPlayerDisconnect
        CEF_OnPlayerDisconnect(playerid, reason);
    #endif

    cef_on_player_disconnect(playerid);

    return 1;
}

#if defined _ALS_OnPlayerDisconnect
    #undef OnPlayerDisconnect
#else
    #define _ALS_OnPlayerDisconnect
#endif

#define OnPlayerDisconnect CEF_OnPlayerDisconnect

#if defined CEF_OnPlayerDisconnect
    forward CEF_OnPlayerDisconnect(playerid, reason);
#endif

#endif
