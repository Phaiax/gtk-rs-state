

#[macro_export]
macro_rules! widget_refs {
    ( $structname:ident: $( $t:ty => $i:ident ),* ) => {

        widget_refs!( ;GET_REFS; $structname: $( $t => $i ),* );
        widget_refs!( ;REF_STRUCT; $structname: $( $t => $i ),* );
        widget_refs!( ;IMPL_GETTERS; $structname: $( $t => $i ),* );


    };
    ( ;GET_REFS; $structname:ident: $( $t:ty => $i:ident ),* ) => {
        use std::cell::RefCell;
        impl From<&gtk::Builder> for $structname {
            fn from(builder: &gtk::Builder) -> WidgetRefs {
                $structname {
                    $($i : RefCell::new(builder.get_object(stringify!($i)).unwrap()), )*
                }
            }
        }
    };
    ( ;REF_STRUCT; $structname:ident: $( $t:ty => $i:ident ),* ) => {
        pub struct $structname {
            $( pub $i : RefCell<$t>, )*
        }
    };
    ( ;IMPL_GETTERS; $structname:ident: $( $t:ty => $i:ident ),* ) => {
        impl $structname {
            $( pub fn $i(&self) -> $t {
                use std::ops::Deref;
                return self.$i.borrow().deref().clone();
            } )*
        }
    };
}


