// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
A simple future type that is always pending.

This is primarily useful for testing or "todo"-style workflows.
*/

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct PendForever;

impl Future for PendForever {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}


//boilerplate
impl Default for PendForever {
    fn default() -> Self {
        PendForever
    }
}

