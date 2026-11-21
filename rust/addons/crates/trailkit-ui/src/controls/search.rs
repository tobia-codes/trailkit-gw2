//! The one search of every trailkit list and page (BL30, BL31, Q581, mock
//! https://claude.ai/artifact/DQRhxQtb4zyUxKB1hBtEVy v9): a search field
//! with the magnifier and an `x` at its end, plus "x of y" with previous and
//! next. A list puts the count in a row below the field; the find bar of an
//! open page (Ctrl+F, memo M198) puts it beside the field and moves it to a
//! second row when the field would get too narrow. Only a closable search
//! (memo's note list, the find bar) shows the `x` on an empty field.
//!
//! The app owns the hits and their order; this module only draws and reports.

pub mod bar;
pub mod field;
