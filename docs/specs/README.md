# Introduction

Datastores, even when very different if pourpose, often follow very similar
approaches to clustering, high availability, failover, etcetera ...

The aim of this specification is to defined a model that details the
attributes, behaviours, and expectations needed of datastores.
After that, we can view and operate on datastores through the model
and expect the same results regardless of the software being manipulated.


## What is a datastore?
A datastore is any software that stores state.
State is some information that, if lost, can't easly be regenerated.
For example:

  * A list of users is state: if lost, users would have to re-register.
  * A cached web page is NOT state: if lost, the page can be fetch from the origin.

This, combined with the fact that individual datastore nodes
store up to several terabytes of data, means that simple
operations may become expensive and/or risky.
