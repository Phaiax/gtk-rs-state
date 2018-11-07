#![allow(dead_code, unused_variables, unused_imports, deprecated)]

use std::thread;

use gtk::prelude::*;

use gtk::{Button, Window, WindowType};

use lazy_static::lazy_static;

use gobject_sys::g_signal_new;

fn main() {
    init_gtk();
}


macro_rules! gtk_refs {
    ( $( $t:ty => $i:ident ),* ) => {
       gtk_refs!( GET_REFS: $( $t => $i ),* );
       gtk_refs!( REF_STRUCT: $( $t => $i ),* );
       gtk_refs!( IMPL_GETTERS: $( $t => $i ),* );
    };
    ( GET_REFS: $( $t:ty => $i:ident ),* ) => {
        use std::cell::RefCell;
        impl From<&gtk::Builder> for GtkRefs {
            fn from(builder: &gtk::Builder) -> GtkRefs {
                GtkRefs {
                    $($i : RefCell::new(builder.get_object(stringify!($i)).unwrap()), )*
                }
            }
        }
    };
    ( REF_STRUCT: $( $t:ty => $i:ident ),* ) => {
        struct GtkRefs {
            $( pub $i : RefCell<$t>, )*
        }
    };
    ( IMPL_GETTERS: $( $t:ty => $i:ident ),* ) => {
        impl GtkRefs {
            $( pub fn $i(&self) -> $t {
                use std::ops::Deref;
                return self.$i.borrow().deref().clone(); } )*
        }
    };
}

gtk_refs!(
    gtk::Window => main_window,
    gtk::Button => button1,
    gtk::Button => button2,
    gtk::Button => button3,
    gtk::Entry => entry1
);

use std::sync::mpsc;
use std::sync::Mutex;
use std::any::Any;

fn with_refs<F>(r:F) where F : FnMut(&GtkRefs) + Send + 'static {
    let tx = TX.lock().unwrap();
    let (back_tx, back_rx) = mpsc::channel();
    tx.as_ref().unwrap().send((Box::new(r), back_tx)).unwrap();
    handle_one();
}

fn handle_one() {
    gtk::idle_add(|| {
        RX.with(|rx| {
            let rx = rx.borrow();
            let mut bc : FnAndBack = rx.as_ref().unwrap().recv().unwrap();
            REFS.with(|refs| {
                let refs = refs.borrow();
                let refs = refs.as_ref().unwrap();
                (*bc.0)(&refs);
            });
        });
        Continue(false)
    });
}

pub fn call_and_pass_borrow(headerbar: &gtk::Entry) {
    // headerbar clone borrow is passed into this function so it has access to headerbar
    headerbar.set_text("This headerbar was changed by call_and_pass_borrow!");
}

pub fn external_element_access() {
    // Direct access is probably not possible?
    // example: headerbar.set_title("Title was set by external function")
}

type FnAndBack = (Box<FnMut(&GtkRefs) + Send>, mpsc::Sender<Box<Any + Send>>);

thread_local!(
    static REFS: RefCell<Option<GtkRefs>> = RefCell::new(None);
    static RX: RefCell<Option<mpsc::Receiver<FnAndBack>>> = RefCell::new(None);
);

lazy_static!(
    static ref TX: Mutex<Option<mpsc::Sender<FnAndBack>>> = Mutex::new(None);
);

pub fn init_gtk() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui.glade");
    let builder = gtk::Builder::new_from_string(glade_src);
    let refs = GtkRefs::from(&builder);

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


    let (tx_fn, rx_fn) = mpsc::channel();
    REFS.with(|refs_| { *refs_.borrow_mut() = Some(refs); });
    RX.with(|rx| { *rx.borrow_mut() = Some(rx_fn); });
    *TX.lock().unwrap() = Some( tx_fn );



    thread::spawn(|| {
        thread::sleep_ms(2000);
        with_refs(|_| {
            println!("with_refs called");
        });
    });



    gtk::main();
}
