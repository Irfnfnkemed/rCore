use crate::sbi::print;


#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::sbi::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::sbi::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}