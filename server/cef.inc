#if !defined CEFSAMP_INC_
	#define CEFSAMP_INC_

	enum CEF_ValueType
	{
		CEF_STRING,
		CEF_INTEGER,
		CEF_FLOAT
	};

	#define CEFSTR(%0) CEF_STRING, %0
	#define CEFINT(%0) CEF_INTEGER, %0
	#define CEFFLOAT(%0) CEF_FLOAT, %0

	#define CEF_DEFAULT_MAX_DIST 50.0
	#define CEF_DEFAULT_REF_DIST 15.0

	native cef_create_browser(player_id, browser_id, const url[], bool:hidden, bool:focused);
	native cef_destroy_browser(player_id, browser_id);
	native cef_on_player_connect(player_id, const ip[]);
	native cef_on_player_disconnect(player_id);
	native cef_emit_event(player_id, const event[], {CEF_ValueType, Float, _}:...);
	native cef_subscribe(const event[], const callback[]);
	native cef_hide_browser(player_id, browser_id, bool:hide);
	native cef_create_ext_browser(player_id, browser_id, const texture[], const url[], scale);
	native cef_append_to_object(player_id, browser_id, object_id);
	native cef_remove_from_object(player_id, browser_id, object_id);
	native cef_player_has_plugin(player_id);
	native cef_toggle_dev_tools(player_id, browser_id, bool:enabled);
	native cef_set_audio_settings(player_id, browser_id, Float:max_distance = CEF_DEFAULT_MAX_DIST, Float:reference_distance = CEF_DEFAULT_REF_DIST);
	native cef_focus_browser(player_id, browser_id, bool:focused);
	native cef_always_listen_keys(player_id, browser_id, bool:listen);
	native cef_load_url(player_id, browser_id, const url[]);

	forward OnCefInitialize(player_id, success);
	forward OnCefBrowserCreated(player_id, browser_id, status_code);

	public OnPlayerConnect(playerid)
	{
		new ip[16];
		GetPlayerIp(playerid, ip, sizeof(ip));
		cef_on_player_connect(playerid, ip);

		#if defined CEF_OnPlayerConnect
			CEF_OnPlayerConnect(playerid);
		#endif

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

	public OnPlayerDisconnect(playerid, reason)
	{
		cef_on_player_disconnect(playerid);

		#if defined CEF_OnPlayerDisconnect
			CEF_OnPlayerDisconnect(playerid, reason);
		#endif

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
#endif // !defined CEFSAMP_INC_
