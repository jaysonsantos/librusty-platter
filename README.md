# librusty-platter

## What is it?
This is supposed to be a folder encryption lib which supports plugabble filesystems (could be local FS, Amazon S3, Dropbox) and plugabble frontends (fuse, http or just as a lib to be embedded). In the end it is supposed to look pretty much like encfs but written in rust because I wanted to avoid basic mistakes (that I would make) with C/C++.

## Warning
This is also working as playground to learn more about encryption and rust so, I wouldn't suggest it for real usage.

## What works
- Local directory initialization
- Folder creation
- Exist method
