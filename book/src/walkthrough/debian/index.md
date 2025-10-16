# Making a Debian liveboot iso with Raptor

In this walkthrough, we will take a look at all the steps needed to make a
customized Debian liveboot `.iso` image.

By putting the image on a USB stick or external hard drive, any computer can be
booted from this external device, without affecting existing data on the
computer.

Using this technique, you will be able to create custom images, containing the
tools, programs and settings that you like, and build them consistently and
easily with raptor.

This guide is presented as a three-step process:

 - [Build](build.md): Build raptor layers suitable for live booting.

 - [Generate](iso.md): Generate a live boot iso from the layers

 - [Make](make.md): Use raptor-make to simplify the build process **[optional]**

~~~admonish tldr
Below is an absolutely minimal example of generating a liveboot iso, for the impatient.
~~~

----

Create a raptor file:

~~~admonish note title="base.rapt"
```raptor
{{#include ../../../example/base.rapt}}
```
~~~

Git clone (or otherwise download) the necessary build container recipes:
```sh
git clone https://github.com/chrivers/raptor-build.git
```

Then use these to build `liveboot.iso`:

```sh
sudo raptor build -I base -O liveboot.iso -L rbuild raptor-build '$rbuild.deblive'
```

----

After this command has run, you should have the file `liveboot.iso` available. Enjoy!

To learn how (and why) this works, read the next chapters of this guide.
