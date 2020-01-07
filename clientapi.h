
    #include <cstdint>
    
    // Отменить продолжение выполнения колбеков события. А так же не отправлять его серверу.
    static const int EXTERNAL_BREAK = 1;
    // Продолжить выполнение. Если никто не отменил его, будет передано серверу.
    static const int EXTERNAL_CONTINUE = 0;
    
    using BrowserReadyCallback = void(*)(uint32_t);
    using EventCallback = int(*)(const char*, cef_list_value_t*);
    
    extern "C" {
        // Проверка на существование браузера в игре.
        bool cef_browser_exists(uint32_t browser);
        // Создан ли браузер и загружен веб-сайт.
        bool cef_browser_ready(uint32_t browser);
        // Создать браузер с указанными параметрами. Эта функция асинхронная, браузер создается не сразу.
        void cef_create_browser(uint32_t id, const char *url, bool hidden, bool focused);
        // Создать CefListValue внутри клиента.
        cef_list_value_t *cef_create_list();
        // Удалить браузер у клиента.
        void cef_destroy_browser(uint32_t id);
        // Вызвать событие у браузера.
        void cef_emit_event(const char *event, cef_list_value_t *list);
        // Сфокусировать ввод на браузере, а так же вывести его поверх всех остальных.
        void cef_focus_browser(uint32_t id, bool focus);
        // Активно ли окно игры.
        bool cef_gta_window_active();
        // Скрыть браузер.
        void cef_hide_browser(uint32_t id, bool hide);
        // Проверить доступен ли ввод для конкретного браузера.
        bool cef_input_available(uint32_t browser);
        // Подписаться на событие полного создания браузера.
        void cef_on_browser_ready(uint32_t browser, BrowserReadyCallback callback);
        bool cef_ready();
        // Подписаться на события от браузера.
        void cef_subscribe(const char *event, EventCallback callback);
        // Попытаться сфокусироваться на браузере. Аналогично паре `cef_input_available` + `cef_focus_browser`,
        // но с одним значительным условием, между выполнением этих двух функции кто-то другой может захватить фокус.
        // А данная функция атомарна, что позволяет проверить и сразу же захватить, гарантируя,
        // что никто другой не сможет в это время получить фокус.
        bool cef_try_focus_browser(uint32_t browser);
    }
