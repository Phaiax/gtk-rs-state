#![allow(dead_code, unused_variables, unused_imports, deprecated)]
#![feature(fnbox)]

mod withgtk;

use std::thread;
use gtk::prelude::*;
use gtk::{Button, Window, WindowType};
use std::time::SystemTime;

// fn main() {
//     init_gtk();
// }



gtk_refs!(
    glade1:
    gtk::Window => main_window,
    gtk::Button => button1,
    gtk::Button => button2,
    gtk::Button => button3,
    gtk::Entry => entry1
);




pub fn call_and_pass_borrow(headerbar: &gtk::Entry) {
    // headerbar clone borrow is passed into this function so it has access to headerbar
    headerbar.set_text("This headerbar was changed by call_and_pass_borrow!");
}

pub fn external_element_access() {
    // Direct access is probably not possible?
    // example: headerbar.set_title("Title was set by external function")
}


gtk_refs!(
    glade2:                         // modulename
    gtk::Window => main_window,     // Widgettype => widget_name_from_glade
    gtk::Entry => entry1
);
fn main() {
    gtk::init().unwrap();
    let glade_src = include_str!("../ui.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    glade2::store_refs(&builder);
    // Optional: You can use the WidgetRefs type as a helper in
    // the main thread for yourself.
    let refs = glade2::WidgetRefs::from(&builder);
    // This type has a function for each of your widgets.
    // These functions return a clone() of the widget.
    refs.main_window().show_all();

    // Start event loop and some other thread
    std::thread::spawn(thread_fn);

    gtk::main();
 }

 fn thread_fn()  {
    glade2::with_refs(|refs| {
        refs.entry1().set_text("Blub!");
    });
 }

pub fn init_gtk() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let refs = glade1::WidgetRefs::from(&builder);

    // main_window and all its elements

    let entry1 = refs.entry1();
    refs.button1().connect_clicked(move |_| {
        entry1.set_text("Button1 was clicked!");
    });

    let entry1 = refs.entry1();
    refs.button2().connect_clicked(move |_| {
        entry1.set_text("Button2 was clicked!");
    });

    let entry1 = refs.entry1();
    refs.button3().connect_clicked(move |_| {
        call_and_pass_borrow(&entry1);
    });

    refs.main_window().connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });



    refs.main_window().show_all();


    glade1::store_refs(&builder);


    // This is an example and a very bad idea in once.
    //
    thread::spawn(|| {

        let start = SystemTime::now();
        let mut loops = 0u64;
        loop {
            loops += 1;
            let elapsed = start.elapsed().unwrap();
            let timedisplay = format!("{}s.{}", elapsed.as_secs(), elapsed.subsec_micros());
            glade1::with_refs(move |refs| {
                refs.entry1().set_text(&timedisplay);
            });
            if loops % 1000 == 0 {
                let elapsed_us = (elapsed.as_secs() * 1000000) + elapsed.subsec_micros() as u64;
                let time_per_loop = elapsed_us as f32 / loops as f32;
                println!("Loop time: {} µs", time_per_loop);
            }
            //zthread::sleep_ms(1);
        }

    });

    thread::spawn(|| {

        let mut temp = 22;

        glade1::with_refs(move |refs| {
            temp += 2;
            //*TEMP.lock().unwrap() = Some(&temp);
            panic!("Boom");
        });

    });

    std::panic::catch_unwind(|| {
        gtk::main();
    }).ok();
    thread::sleep_ms(100);
}

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static!(
    /// This is the sender part of the above RX channel. All non-gtk
    /// threads can put their UiActions-Closures here and wait for
    /// completion afterwards.
    static ref TEMP: Mutex<Option<&'static u32>> = Mutex::new(None);
);

