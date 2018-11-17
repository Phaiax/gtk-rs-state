

#[macro_export]
macro_rules! widget_refs {
    ( $structname:ident ; $( $i:ident : $t:ty),* ) => {

        widget_refs!( ;GET_REFS; $structname ; $( $i : $t ),* );
        widget_refs!( ;REF_STRUCT; $structname ; $( $i : $t ),* );
        widget_refs!( ;IMPL_GETTERS; $structname ; $( $i : $t ),* );


    };
    ( ;GET_REFS; $structname:ident ; $( $i:ident : $t:ty ),* ) => {
        impl<'a> From<&'a gtk::Builder> for $structname {
            fn from(builder: &gtk::Builder) -> WidgetRefs {
                $structname {
                    $($i : builder.get_object(stringify!($i)).unwrap(), )*
                }
            }
        }
    };
    ( ;REF_STRUCT; $structname:ident ; $( $i:ident : $t:ty ),* ) => {
        pub struct $structname {
            $( pub $i : $t, )*
        }
    };
    ( ;IMPL_GETTERS; $structname:ident ; $( $i:ident : $t:ty ),* ) => {
        impl $structname {
            $( pub fn $i(&self) -> $t {
                return self.$i.clone();
            } )*
        }
    };
}


