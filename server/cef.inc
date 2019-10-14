#if !defined VOICECHAT_INC

#define VOICECHAT_INC

#define VOICE_CHAT_GLOBAL 1
#define VOICE_CHAT_TEAM 2
#define VOICE_CHAT_ALL 3

// internal
native voice_on_player_connect(playerid, const ip[]);
native voice_on_player_disconnect(playerid);
native voice_stream_in(playerid, forplayerid);
native voice_stream_out(playerid, forplayerid);

// checks, все функции возвращают bool
native voice_player_has_plugin(player_id);
native voice_player_has_microphone(player_id);

// voice control
native voice_mute_player(player_id, chat);
native voice_deafen_player(player_id, chat);
native voice_unmute_player(player_id, chat);
native voice_undeafen_player(player_id, chat);

// channels
native voice_clear_player_channel(player_id);
native voice_add_player_in_channel(player_id, channel_id);
native voice_delete_channel(channel_id);
native voice_create_channel();


// internal callbacks hooks

// Connect
public OnPlayerConnect(playerid)
{
    #if defined VoiceChat_OnPlayerConnect
        VoiceChat_OnPlayerConnect(playerid);
    #endif

    new ip_port[22];
    GetPlayerIp(playerid, ip_port, sizeof(ip_port));
    voice_on_player_connect(playerid, ip_port);

    return 1;

}
#if defined _ALS_OnPlayerConnect
    #undef OnPlayerConnect
#else
    #define _ALS_OnPlayerConnect
#endif

#define OnPlayerConnect VoiceChat_OnPlayerConnect

#if defined VoiceChat_OnPlayerConnect
    forward VoiceChat_OnPlayerConnect(playerid);
#endif

// Disconnect
public OnPlayerDisconnect(playerid, reason)
{
    #if defined VoiceChat_OnPlayerDisconnect
        VoiceChat_OnPlayerDisconnect(playerid, reason);
    #endif

    voice_on_player_disconnect(playerid);

    return 1;
}

#if defined _ALS_OnPlayerDisconnect
    #undef OnPlayerDisconnect
#else
    #define _ALS_OnPlayerDisconnect
#endif

#define OnPlayerDisconnect VoiceChat_OnPlayerDisconnect

#if defined VoiceChat_OnPlayerDisconnect
    forward VoiceChat_OnPlayerDisconnect(playerid, reason);
#endif

// Stream In
public OnPlayerStreamIn(playerid, forplayerid)
{
    #if defined VoiceChat_OnPlayerStreamIn
        VoiceChat_OnPlayerStreamIn(playerid, forplayerid);
    #endif

    voice_stream_in(playerid, forplayerid);

    return 1;
}

#if defined _ALS_OnPlayerStreamIn
    #undef OnPlayerStreamIn
#else
    #define _ALS_OnPlayerStreamIn
#endif

#define OnPlayerStreamIn VoiceChat_OnPlayerStreamIn

#if defined VoiceChat_OnPlayerStreamIn
    forward VoiceChat_OnPlayerStreamIn(playerid, forplayerid);
#endif

// Stream Out
public OnPlayerStreamOut(playerid, forplayerid)
{
    #if defined VoiceChat_OnPlayerStreamOut
        VoiceChat_OnPlayerStreamOut(playerid, forplayerid);
    #endif

    voice_stream_out(playerid, forplayerid);

    return 1;
}

#if defined _ALS_OnPlayerStreamOut
    #undef OnPlayerStreamOut
#else
    #define _ALS_OnPlayerStreamOut
#endif

#define OnPlayerStreamOut VoiceChat_OnPlayerStreamOut

#if defined VoiceChat_OnPlayerStreamOut
    forward VoiceChat_OnPlayerStreamOut(playerid, forplayerid);
#endif

#endif
