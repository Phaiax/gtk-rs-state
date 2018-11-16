#![allow(dead_code, unused_variables, unused_imports, deprecated)]
#![feature(fnbox)]

use std::thread;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{Button, Window, WindowType};

use gtk_rs_state::gtk_refs;

gtk_refs!(
    widgets:                         // modulename
    gtk::Window => main_window,     // Widgettype => widget_name_from_glade
    gtk::Button => button1
);

fn main() {

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("gtk-rs-state Example Program");
    window.set_default_size(350, 70);
    let button = Button::new_with_label("Spawn another thread!");
    window.add(&button);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    button.connect_clicked(|_| {
        std::thread::spawn(external_element_access);
        println!("Clicked!");
    });

    // You need the following two statements to prepare the
    // static storage needed for cross thread access.
    // See the `from_glade.rs` example for a more elegant solution
    let widget_references = widgets::WidgetRefs {
        main_window: RefCell::new(window.clone()),
        button1:     RefCell::new(button.clone()),
    };

    widgets::store_refs(widget_references);
    // En

    // This type has a function for each of your widgets.
    // These functions return a clone() of the widget.
    window.show_all();

    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Start event loop and some other thread

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
        let text = format!("Round {} in {:?}", i, std::thread::current().id());

        widgets::with_refs(|refs| {
            refs.button1().set_label(&text);
        });
    }
 }

