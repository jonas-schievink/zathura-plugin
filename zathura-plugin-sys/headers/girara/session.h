/* SPDX-License-Identifier: Zlib */

#ifndef GIRARA_SESSION_H
#define GIRARA_SESSION_H

#include "types.h"
#include "macros.h"
#include "callbacks.h"

#include <gtk/gtk.h>

#ifdef GDK_WINDOWING_X11
#include <gtk/gtkx.h>
#else
typedef int Window;
#endif

struct girara_session_s
{
  struct
  {
    GtkWidget       *window; /**< The main window of the application */
    GtkBox          *box; /**< A box that contains all widgets */
    GtkWidget       *view; /**< The view area of the applications widgets */
    GtkWidget       *viewport; /**< The viewport of view */
    GtkWidget       *statusbar; /**< The statusbar */
    GtkBox          *statusbar_entries; /**< Statusbar entry box */
    GtkWidget       *notification_area; /**< The notification area */
    GtkWidget       *notification_text; /**< The notification entry */
    GtkBox          *inputbar_box; /**< Inputbar box */
    GtkWidget       *inputbar; /**< Inputbar event box */
    GtkLabel        *inputbar_dialog; /**< Inputbar dialog */
    GtkEntry        *inputbar_entry; /**< Inputbar entry */
    GtkBox          *results; /**< Completion results */
    Window          embed; /**< Embedded window */
  } gtk;

  struct
  {
    girara_list_t* mouse_events; /**< List of mouse events */
    girara_list_t* commands; /**< List of commands */
    girara_list_t* shortcuts; /**< List of shortcuts */
    girara_list_t* special_commands; /**< List of special commands */
    girara_list_t* inputbar_shortcuts; /**< List of inputbar shortcuts */
  } bindings;

  struct
  {
    int inputbar_activate; /**< Inputbar activation */
    int inputbar_key_pressed; /**< Pressed key in inputbar */
    int inputbar_changed; /**< Inputbar text changed */
    int view_key_pressed; /**< Pressed key in view */
    int view_button_press_event; /**< Pressed button */
    int view_button_release_event; /**< Released button */
    int view_motion_notify_event; /**< Cursor movement event */
    int view_scroll_event; /**< Scroll event */
    girara_callback_inputbar_activate_t inputbar_custom_activate; /**< Custom handler */
    girara_callback_inputbar_key_press_event_t inputbar_custom_key_press_event; /**< Custom handler */
    void* inputbar_custom_data; /**< Data for custom handler */
  } signals;

  struct
  {
    void (*buffer_changed)(girara_session_t* session); /**< Buffer changed */
    bool (*unknown_command)(girara_session_t* session, const char* input); /**< Unknown command */
  } events;

  struct
  {
    GString *buffer; /**< Buffer */
    void* data; /**< User data */
    bool autohide_inputbar; /**< Auto-hide inputbar */
    bool hide_statusbar; /**< Hide statusbar */
  } global;

  struct
  {
    girara_mode_t current_mode; /**< Current mode */
    girara_list_t *identifiers; /**< List of modes with its string identifiers */
    girara_mode_t normal; /**< The normal mode */
    girara_mode_t inputbar; /**< The inputbar mode */
  } modes;

  GiraraInputHistory* command_history; /**< Command history */
  girara_session_private_t* private_data; /**< Private data of a girara session */
};

/**
 * Creates a girara session
 *
 * @return A valid session object
 * @return NULL when an error occurred
 */
girara_session_t* girara_session_create(void) GIRARA_VISIBLE;

/**
 * Initializes an girara session
 *
 * @param session The used girara session
 * @param appname Name of the session (can be NULL)
 * @return TRUE No error occurred
 * @return FALSE An error occurred
 */
bool girara_session_init(girara_session_t* session, const char* appname) GIRARA_VISIBLE;

/**
 * Destroys an girara session
 *
 * @param session The used girara session
 * @return TRUE No error occurred
 * @return FALSE An error occurred
 */
bool girara_session_destroy(girara_session_t* session) GIRARA_VISIBLE;

/**
 * Sets the view widget of girara
 *
 * @param session The used girara session
 * @param widget The widget that should be displayed
 * @return TRUE No error occurred
 * @return FALSE An error occurred
 */
bool girara_set_view(girara_session_t* session, GtkWidget* widget) GIRARA_VISIBLE;

/**
 * Returns a copy of the buffer
 *
 * @param session The used girara session
 * @return Copy of the current buffer
 */
char* girara_buffer_get(girara_session_t* session) GIRARA_VISIBLE;

/**
 * Displays a notification popup for the user using libnotify. Basic styling
 * is allowed using Pango's markup format:
 * <https://developer.gnome.org/pango/stable/PangoMarkupFormat.html>
 *
 * @param session The girara session
 * @param summary The title
 * @param body The content
 */
void girara_libnotify(girara_session_t* session, const char *summary,
    const char *body) GIRARA_VISIBLE;

/**
 * Displays a notification for the user. It is possible to pass GIRARA_INFO,
 * GIRARA_WARNING or GIRARA_ERROR as a notification level.
 *
 * @param session The girara session
 * @param level The level
 * @param format String format
 * @param ...
 */
void girara_notify(girara_session_t* session, int level,
    const char* format, ...) GIRARA_PRINTF(3, 4) GIRARA_VISIBLE;

/**
 * Creates a girara dialog
 *
 * @param session The girara session
 * @param dialog The dialog message
 * @param invisible Sets the input visibility
 * @param key_press_event Callback function to a custom key press event handler
 * @param activate_event Callback function to a custom activate event handler
 * @param data Custom data that is passed to the callback functions
 */
void girara_dialog(girara_session_t* session, const char* dialog, bool
    invisible, girara_callback_inputbar_key_press_event_t key_press_event,
    girara_callback_inputbar_activate_t activate_event, void* data) GIRARA_VISIBLE;

/**
 * Adds a new mode by its string identifier
 *
 * @param session The used girara session
 * @param name The string identifier used in configs/inputbar etc to refer by
 * @return A newly defined girara_mode_t associated with name
 */
girara_mode_t girara_mode_add(girara_session_t* session, const char* name) GIRARA_VISIBLE;

/**
 * Sets the current mode
 *
 * @param session The used girara session
 * @param mode The new mode
 */
void girara_mode_set(girara_session_t* session, girara_mode_t mode) GIRARA_VISIBLE;

/**
 * Returns the current mode
 *
 * @param session The used girara session
 * @return The current mode
 */
girara_mode_t girara_mode_get(girara_session_t* session) GIRARA_VISIBLE;

/**
 * Set name of the window title
 *
 * @param session The used girara session
 * @param name The new name of the session
 * @return true if no error occurred
 * @return false if an error occurred
 */
bool girara_set_window_title(girara_session_t* session, const char* name) GIRARA_VISIBLE;

/**
 * Set icon of the window
 *
 * @param session The used girara session
 * @param name the name of the themed icon
 * @return true if no error occurred
 * @return false if an error occurred
 */
bool girara_set_window_icon(girara_session_t* session, const char* name) GIRARA_VISIBLE;

/**
 * Returns the command history
 *
 * @param session The used girara session
 * @return The command history (list of strings) or NULL
 */
girara_list_t* girara_get_command_history(girara_session_t* session) GIRARA_VISIBLE;

/**
 * Returns the internal template object to apply custom theming options
 *
 * @param session The girara session
 * @returns GiraraTemplate object
 */
GiraraTemplate* girara_session_get_template(girara_session_t* session) GIRARA_VISIBLE;

/**
 * Replaces the internal template object, thus provides entirely user-defined styling.
 *
 * @param session The girara session
 * @param template The template to apply, @ref girara_template_new
 * @param init_variables Defines whether the default variables and current
 *    values should be added to the the template
 *
 * @note Using the template @c girara_template_new("") will use the default gtk style
 *
 */
void girara_session_set_template(girara_session_t* session, GiraraTemplate* template, bool init_variables) GIRARA_VISIBLE;

#endif
