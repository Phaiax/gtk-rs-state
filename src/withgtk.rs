

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

            use std::sync::mpsc;
            use std::sync::Mutex;
            use std::any::Any;
            use std::boxed::FnBox;
            use glib::source::idle_add;
            use gtk::Continue;

            // macros
            use super::gtk_refs;
            use lazy_static::lazy_static;

            gtk_refs!( ;GET_REFS; $( $t => $i ),* );
            gtk_refs!( ;REF_STRUCT; $( $t => $i ),* );
            gtk_refs!( ;IMPL_GETTERS; $( $t => $i ),* );

            type BoxedUiAction = Box<FnBox(&WidgetRefs) + Send + 'static>;
            type FnAndEvent = (BoxedUiAction, mpsc::Sender<()>);


            thread_local!(
                /// This variable is only populated in the gtk thread.
                /// The inner struct contains a reference counted
                /// pointer to all selected widgets.
                static REFS: RefCell<Option<WidgetRefs>> = RefCell::new(None);
                /// This variable is only populated in the gtk thread.
                /// The gtk thread receives the boxed UiActions-Closures via
                /// this channel. Additionaly, another channels sender is
                /// transfered in this channel. This sender is used as signal
                /// so the non-gtk thread can wait for completion.
                static RX: RefCell<Option<mpsc::Receiver<FnAndEvent>>> = RefCell::new(None);
            );

            lazy_static!(
                /// This is the sender part of the above RX channel. All non-gtk
                /// threads can put their UiActions-Closures here and wait for
                /// completion afterwards.
                static ref TX: Mutex<Option<mpsc::Sender<FnAndEvent>>> = Mutex::new(None);
            );


            /// Initializes the static storages.
            /// You must call this function before calling `with_refs()`.
            /// See `withgtk` module documentation for examples.
            pub fn store_refs(builder: &gtk::Builder) {
                if TX.lock().unwrap().is_some() {
                    panic!("You must only call store_refs() once!");
                }

                let refs : WidgetRefs = builder.into();
                let (uiactions_tx, uiactions_rx) = mpsc::channel();
                REFS.with(|refs_| { *refs_.borrow_mut() = Some(refs); });
                RX.with(|rx| { *rx.borrow_mut() = Some(uiactions_rx); });
                *TX.lock().unwrap() = Some( uiactions_tx );
            }

            /// Call this from wherever you want (especially from non-gtk threads).
            /// The closure argument allows you to modify the ui.
            pub fn with_refs<'a, F>(uiaction: F) where F : FnOnce(&WidgetRefs) + Send + 'a {
                let uiaction : Box<FnBox(&WidgetRefs) + Send + 'a> = Box::new(uiaction);
                // Extend the livetime to be static
                // I think this should work, because we use the `event_callback_finished_..`
                // mechanism to wait until the closure has finished executing.
                // I tested leaking thread stack variables and I was not successful.
                // But maybe I was not creative enough.
                // If you don't trust me, remove this statement and change the lifetime
                // 'a to static in this function definition.
                let uiaction = unsafe {
                     std::mem::transmute::
                        <Box<FnBox(&WidgetRefs) + Send + 'a>,
                         Box<FnBox(&WidgetRefs) + Send + 'static>>(uiaction)
                };
                let uiactions_tx = TX.lock().unwrap();
                let (event_callback_finished_tx,
                     event_callback_finished_rx) = mpsc::channel();
                uiactions_tx.as_ref()
                            .expect("Please call store_refs() in the gtk thread before using with_refs()")
                            .send((uiaction, event_callback_finished_tx))
                            .expect("Gtk thread seems to have panicked!");

                // Notify the gtk event loop and perform the uiaction
                handle_one_callback_in_gtk_thread();
                // wait until gtk thread has executed the closure
                event_callback_finished_rx.recv().expect("withgtk: with_refs: The closure has paniced while executing!");
            }

            fn handle_one_callback_in_gtk_thread() {
                idle_add(|| {
                    RX.with(|uiactions_rx| {
                        let uiactions_rx = uiactions_rx.borrow();
                        // The first unwrap happens when the user forgot to initialize
                        // but since with_refs just successfully unpacked the uiactions_tx part
                        // this can never fail.
                        // The second unwrap(/expect) happens if there are no more items in the queue
                        // and all senders have disconnected.
                        // The first can not happen since
                        // with_refs just added an item, the second cannot happen because
                        // the sender is a not thread-local static which we never clear
                        // (at least in the current implementation).
                        // There is a possible race condition if the user calls
                        // store_refs multiple times and some thread uses an old sender
                        // while we swap the sender. The callback that this thread issues
                        // will then use the new receiver which has no elements in it.
                        // Then this will wait indefinitly.
                        let bc : FnAndEvent = uiactions_rx.as_ref().unwrap()
                            .try_recv()
                            .expect("Race condition! I guess you called `store_refs` more than once!");
                        let (uiaction, finished_callback) = bc;
                        // Call the uiaction with the references to all the gtk widgets.
                        // REFS is Some() because we are called in the gtk thread.
                        REFS.with(|refs| {
                            let refs = refs.borrow();
                            let refs = refs.as_ref()
                                           .expect("Please call store_refs() in the gtk thread before using with_refs()");
                            uiaction.call_box((refs, ));
                        });
                        finished_callback.send(()).unwrap();
                    });
                    Continue(false)
                });
            }


        }
    };
    ( ;GET_REFS; $( $t:ty => $i:ident ),* ) => {
        use std::cell::RefCell;
        impl From<&gtk::Builder> for WidgetRefs {
            fn from(builder: &gtk::Builder) -> WidgetRefs {
                WidgetRefs {
                    $($i : RefCell::new(builder.get_object(stringify!($i)).unwrap()), )*
                }
            }
        }
    };
    ( ;REF_STRUCT; $( $t:ty => $i:ident ),* ) => {
        pub struct WidgetRefs {
            $( pub $i : RefCell<$t>, )*
        }
    };
    ( ;IMPL_GETTERS; $( $t:ty => $i:ident ),* ) => {
        impl WidgetRefs {
            $( pub fn $i(&self) -> $t {
                use std::ops::Deref;
                return self.$i.borrow().deref().clone();
            } )*
        }
    };
}


