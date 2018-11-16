

pub mod fnbox;
pub mod withgtk;
pub mod widgetrefs;

#[macro_export]
macro_rules! gtk_refs {
    ( $name:ident: $( $t:ty => $i:ident ),* ) => {


        ///
        /// # Usage
        ///
        /// ```
        /// use std::thread;
        /// use gtk::prelude::*;
        /// mod withgtk;
        ///
        /// gtk_refs!(
        ///     glade1:                         // modulename
        ///     gtk::Window => main_window,     // Widgettype => widget_name_from_glade
        ///     gtk::Entry => entry1
        /// );
        ///
        /// fn main() {
        ///     gtk::init().unwrap();
        ///     let glade_src = include_str!("ui.glade");
        ///     let builder = gtk::Builder::new_from_string(glade_src);
        ///     glade1::store_refs(&builder);
        ///
        ///     // Optional: You can use the WidgetRefs type as a helper in
        ///     // the main thread for yourself.
        ///     let refs = glade1::WidgetRefs::from(&builder);
        ///
        ///     // This type has a function for each of your widgets.
        ///     // These functions return a clone() of the widget.
        ///     refs.main_window().show_all();
        ///
        ///     // Start event loop and some other thread
        ///     std::thread::spawn(thread_fn);
        ///     gtk::main();
        ///  }
        ///
        ///  fn thread_fn()  {
        ///     glade1::with_refs(|refs| {
        ///         refs.entry1().set_text("Blub!");
        ///     });
        ///  }
        ///
        ///
        ///
        pub mod $name {

            use $crate::*;

            widget_refs!(WidgetRefs: $( $t => $i ),* );
            with_gtk!(WidgetRefs);

            pub fn init_storage_from_builder(builder: &gtk::Builder) {
                let refs : WidgetRefs = builder.into();
                init_storage(refs);
            }
        }
    }
}