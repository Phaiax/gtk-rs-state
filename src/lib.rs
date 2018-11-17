
extern crate glib;
extern crate gtk;
extern crate lazy_static;

pub mod fnbox;
pub mod withgtk;
pub mod widgetrefs;

pub mod _modexport {
    pub use glib::source::idle_add;
    pub use gtk::Continue;
    pub use lazy_static::lazy_static;
}

#[macro_export]
macro_rules! gtk_refs {
    ( $modpub:vis mod $modname:ident ; struct $structname:ident ; $( $i:ident : $t:ty ),* ) => {


        ///
        /// # Usage
        ///
        /// ```
        /// gtk_refs!(
        ///     pub mod widgets;                // The macro emits a new module with this name
        ///     struct WidgetRefs;              // The macro emits a struct with this name containing:
        ///     main_window : gtk::Window ,     // widget_name : Widgettype
        ///     button1 : gtk::Button           // ..
        /// );
        ///
        /// fn main() {
        ///
        ///     if gtk::init().is_err() {
        ///         println!("Failed to initialize GTK.");
        ///         return;
        ///     }
        ///
        ///     let window = Window::new(WindowType::Toplevel);
        ///     window.set_title("gtk-fnonce-on-eventloop Example Program");
        ///     window.set_default_size(350, 70);
        ///     let button = Button::new_with_label("Spawn another thread!");
        ///     window.add(&button);
        ///     window.show_all();
        ///
        ///     window.connect_delete_event(|_, _| {
        ///         gtk::main_quit();
        ///         Inhibit(false)
        ///     });
        ///
        ///     button.connect_clicked(|_| {
        ///         std::thread::spawn(some_workfunction);
        ///         println!("Clicked!");
        ///     });
        ///
        ///     // You need the following two statements to prepare the
        ///     // static storage needed for cross thread access.
        ///     // See the `from_glade.rs` example for a more elegant solution
        ///     let widget_references = widgets::WidgetRefs {
        ///         main_window: window.clone(),
        ///         button1:     button.clone(),
        ///     };
        ///
        ///     widgets::init_storage(widget_references);
        ///     // End
        ///
        ///     // This type has a function for each of your widgets.
        ///     // These functions return a clone() of the widget.
        ///     window.show_all();
        ///
        ///     window.connect_delete_event(move |_, _| {
        ///         gtk::main_quit();
        ///         Inhibit(false)
        ///     });
        ///
        ///     // Start event loop
        ///     gtk::main();
        /// }
        ///
        /// fn compute() {
        ///     use std::thread::sleep;
        ///     use std::time::Duration;
        ///     sleep(Duration::from_secs(1));
        /// }
        ///
        /// fn some_workfunction()  {
        ///     let mut i = 0;
        ///
        ///     loop {
        ///         compute();
        ///
        ///         i += 1;
        ///         let text = format!("Round {} in {:?}", i, std::thread::current().id());
        ///
        ///         widgets::do_in_gtk_eventloop(|refs| {
        ///             refs.button1().set_label(&text);
        ///         });
        ///     }
        /// }
        /// ```
        ///
        ///
        $modpub mod $modname {
            use $crate::*;

            widget_refs!($structname; $( $i : $t ),* );
            with_gtk!($structname);

            pub fn init_storage_from_builder(builder: &gtk::Builder) {
                let refs : $structname = builder.into();
                init_storage(refs);
            }
        }
    }
}