#![allow(dead_code, unused_variables, unused_imports, deprecated)]
#![feature(fnbox)]

use std::thread;
use gtk::prelude::*;

use gtk_rs_state::gtk_refs;

gtk_refs!(
    widgets:                         // modulename
    gtk::Window => main_window,     // Widgettype => widget_name_from_glade
    gtk::Button => button1,
    gtk::Button => button2,
    gtk::Button => button3,
    gtk::Entry => entry1
);

fn main() {
    gtk::init().unwrap();
    let glade_src = include_str!("../ui.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    widgets::store_refs_from_builder(&builder);

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
    std::thread::spawn(external_element_access);

    gtk::main();
}


fn compute() {
    use std::thread::sleep;
    use std::time::Duration;
    sleep(Duration::from_secs(1));
}

fn external_element_access()  {
    let mut i = 0;

    loop {
        compute();

        i += 1;
        let text = format!("Round {}", i);

        widgets::with_refs(|refs| {
            refs.entry1().set_text(&text);
        });
    }
 }

