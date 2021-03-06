// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(struct_variant)]

use std::task;

#[deriving(PartialEq, Show)]
enum Message {
    Dropped,
    DestructorRan
}

struct SendOnDrop {
    sender: Sender<Message>
}

impl Drop for SendOnDrop {
    fn drop(&mut self) {
        self.sender.send(Message::Dropped);
    }
}

enum Foo {
    SimpleVariant(Sender<Message>),
    NestedVariant(Box<uint>, SendOnDrop, Sender<Message>),
    FailingVariant { on_drop: SendOnDrop }
}

impl Drop for Foo {
    fn drop(&mut self) {
        match self {
            &Foo::SimpleVariant(ref mut sender) => {
                sender.send(Message::DestructorRan);
            }
            &Foo::NestedVariant(_, _, ref mut sender) => {
                sender.send(Message::DestructorRan);
            }
            &Foo::FailingVariant { .. } => {
                panic!("Failed");
            }
        }
    }
}

pub fn main() {
    let (sender, receiver) = channel();
    {
        let v = Foo::SimpleVariant(sender);
    }
    assert_eq!(receiver.recv(), Message::DestructorRan);
    assert_eq!(receiver.recv_opt().ok(), None);

    let (sender, receiver) = channel();
    {
        let v = Foo::NestedVariant(box 42u, SendOnDrop { sender: sender.clone() }, sender);
    }
    assert_eq!(receiver.recv(), Message::DestructorRan);
    assert_eq!(receiver.recv(), Message::Dropped);
    assert_eq!(receiver.recv_opt().ok(), None);

    let (sender, receiver) = channel();
    task::spawn(proc() {
        let v = Foo::FailingVariant { on_drop: SendOnDrop { sender: sender } };
    });
    assert_eq!(receiver.recv(), Message::Dropped);
    assert_eq!(receiver.recv_opt().ok(), None);

    let (sender, receiver) = channel();
    {
        task::spawn(proc() {
            let mut v = Foo::NestedVariant(box 42u, SendOnDrop {
                sender: sender.clone()
            }, sender.clone());
            v = Foo::NestedVariant(box 42u, SendOnDrop { sender: sender.clone() }, sender.clone());
            v = Foo::SimpleVariant(sender.clone());
            v = Foo::FailingVariant { on_drop: SendOnDrop { sender: sender } };
        });
    }
    assert_eq!(receiver.recv(), Message::DestructorRan);
    assert_eq!(receiver.recv(), Message::Dropped);
    assert_eq!(receiver.recv(), Message::DestructorRan);
    assert_eq!(receiver.recv(), Message::Dropped);
    assert_eq!(receiver.recv(), Message::DestructorRan);
    assert_eq!(receiver.recv(), Message::Dropped);
    assert_eq!(receiver.recv_opt().ok(), None);
}
