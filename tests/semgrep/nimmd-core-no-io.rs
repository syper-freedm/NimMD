// ruleid: nimmd-core-no-io
use std::fs;
// ruleid: nimmd-core-no-io
use std::{fs, io};
// ruleid: nimmd-core-no-io
use std::{io, net::TcpStream};
// ruleid: nimmd-core-no-io
use std::{io::{self, Read}, process::Command};
// ruleid: nimmd-core-no-io
fn spawn() { std::process::exit(0) }
// ruleid: nimmd-core-no-io
fn read() { let _ = std :: fs::read("x"); }
// ruleid: nimmd-core-no-io
use std as s;
// ruleid: nimmd-core-no-io
extern crate std as s;
// ruleid: nimmd-core-no-io
use std::*;
// ruleid: nimmd-core-no-io
use std::{fmt, self as s};
// ok: nimmd-core-no-io
use std::{borrow::Cow, fmt};
// ok: nimmd-core-no-io
use std::io::Write;
// ok: nimmd-core-no-io
use std::fmt::{self, Write};
// ok: nimmd-core-no-io
use std::{io::{self, Read}, fmt};
