
extern crate gtk;
extern crate gtk_rs_state;

use std::rc::Rc;
use std::cell::RefCell;


use gtk::prelude::*;
use gtk::{Button, Window, WindowType};



pub struct Refs {
    button: gtk::Button,
    // This is possible too
    other_non_send_state : Rc<RefCell<Vec<String>>>
}


// This uses the macro with_gtk! and not gtk_refs!() as in the other examples
// Better place the macro with_gtk! in a module, it pollutes its souroundings a bit
mod r {
    use super::Refs;
    use gtk_rs_state::with_gtk;
    /*  // This macro emits the following public elements:

        pub fn init_storage(&Refs);
        pub fn do_in_gtk_eventloop( FnOnce(Rc<Refs>) );

        // And some private stuff which you must ignore
        use std::...;
        use gtk_rs_state::...;
        type BoxedUiAction = ...;
        type FnAndEvent = ...;
        static (threadlocal) DATA = RefCell<Option<Rc<Refs>>>;
        static (threadlocal) RX = RefCell<Option<Receiver<FnAndEvent>>;
        static               TX = RefCell<Option<Sender<FnAndEvent>>;
        fn handle_one_callback_in_gtk_thread();
    */
    with_gtk!(Refs);
}

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
        std::thread::spawn(some_workfunction);
        println!("Clicked!");
    });

    // You need the following two statements to prepare the
    // static storage needed for cross thread access.
    let widget_references = Refs {
        button,
        other_non_send_state : Rc::new(RefCell::new(vec!["Hi!".to_owned()]))
    };
    r::init_storage(widget_references);

    // This type has a function for each of your widgets.
    // These functions return a clone() of the widget.
    window.show_all();

    window.connect_delete_event(move |_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Start event loop
    gtk::main();
}

fn compute() {
    use std::thread::sleep;
    use std::time::Duration;
    sleep(Duration::from_secs(1));
}

fn some_workfunction()  {
    let mut i = 0;

    loop {
        compute();

        i += 1;
        let text = format!("Round {} in {:?}", i, std::thread::current().id());

        r::do_in_gtk_eventloop(|refs| {
            refs.button.set_label(&text);
            refs.other_non_send_state.borrow_mut().push(text.to_owned());
        });
    }
 }

