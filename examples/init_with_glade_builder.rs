
extern crate gtk;
extern crate gtk_fnonce_on_eventloop;

use gtk::prelude::*;
use gtk_fnonce_on_eventloop::gtk_refs;


/* This macro emits the following elements:
    mod widgets {
        pub struct WidgetRefs {
            pub main_window : gtk::Window,
            ...
        }
        impl From<&gtk::Builder> for WidgetRefs { ... };
        impl WidgetRefs {
            fn main_window() -> gtk::Window { }
            ...
        }

        pub fn init_storage(WidgetRefs);
        pub fn init_storage_from_builder(&gtk::Builder);
        pub fn do_in_gtk_eventloop( FnOnce(Rc<WidgetRefs>) );
    }
*/
gtk_refs!(
    pub mod widgets;                // The macro emits a new module with this name
    struct WidgetRefs;              // The macro emits a struct with this name containing:
    main_window : gtk::Window,      // widget_name_from_glade : Widgettype
    button1 : gtk::Button,
    button2 : gtk::Button,
    button3 : gtk::Button,
    entry1 : gtk::Entry
);

fn main() {
    gtk::init().unwrap();
    let glade_src = include_str!("../ui.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    widgets::init_storage_from_builder(&builder);

    // Optional: You can use the WidgetRefs type as a helper in
    // the main thread for yourself.
    let refs = widgets::WidgetRefs::from(&builder);

    // This type has a function for each of your widgets.
    // These functions return a clone() of the widget.
    refs.main_window().show_all();

    refs.main_window().connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Start event loop and some other thread
    std::thread::spawn(some_other_thread);

    gtk::main();
}


fn compute() {
    use std::thread::sleep;
    use std::time::Duration;
    sleep(Duration::from_secs(1));
}

fn some_other_thread()  {
    let mut i = 0;

    loop {
        compute();

        i += 1;
        let text = format!("Round {}", i);

        widgets::do_in_gtk_eventloop(|refs| {
            refs.entry1().set_text(&text);
        });
    }
 }

