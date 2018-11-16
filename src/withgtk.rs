

#[macro_export]
macro_rules! with_gtk {
    ( $struct:ident ) => {



        use std::sync::mpsc;
        use std::sync::Mutex;
        use std::rc::Rc;
        use std::any::Any;
        use glib::source::idle_add;
        use gtk::Continue;
        use $crate::fnbox::SendBoxFnOnce;
        use $crate::fnbox::FnBox;

        // macros
        use super::gtk_refs;
        use lazy_static::lazy_static;


        type BoxedUiAction = SendBoxFnOnce<'static, (Rc<$struct>, )>;
        type FnAndEvent = (BoxedUiAction, mpsc::Sender<()>);


        thread_local!(
            /// This variable is only populated in the gtk thread.
            /// The inner struct contains a reference counted
            /// pointer to all selected widgets.
            static REFS: RefCell<Option<Rc<$struct>>> = RefCell::new(None);
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
        pub fn init_storage(data: $struct) {
            if TX.lock().unwrap().is_some() {
                panic!("You must only call init_storage() once!");
            }

            let (uiactions_tx, uiactions_rx) = mpsc::channel();
            REFS.with(|refs_| { *refs_.borrow_mut() = Some(Rc::new(data)); });
            RX.with(|rx| { *rx.borrow_mut() = Some(uiactions_rx); });
            *TX.lock().unwrap() = Some( uiactions_tx );
        }



        /// Call this from wherever you want (especially from non-gtk threads).
        /// The closure argument allows you to modify the ui.
        pub fn do_in_gtk_eventloop<'a, F>(uiaction: F) where F : FnOnce(Rc<$struct>) + Send + 'a {
            let uiaction : SendBoxFnOnce<'a, (Rc<$struct>, )> = SendBoxFnOnce::from(uiaction);
            // Extend the livetime to be static
            // I think this should work, because we use the `event_callback_finished_..`
            // mechanism to wait until the closure has finished executing.
            // I tested leaking thread stack variables and I was not successful.
            // But maybe I was not creative enough.
            // If you don't trust me, remove this statement and change the lifetime
            // 'a to static in this function definition.
            let uiaction = unsafe {
                 std::mem::transmute::
                    <SendBoxFnOnce<'a, (Rc<$struct>, )>,
                     SendBoxFnOnce<'static, (Rc<$struct>, )>>(uiaction)
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
                        let refs : Rc<$struct> = refs.clone();
                        uiaction.call(refs);
                    });
                    finished_callback.send(()).unwrap();
                });
                Continue(false)
            });
        }


    };
}


