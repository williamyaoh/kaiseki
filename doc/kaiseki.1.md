# NAME

kaiseki - unintrusive literate programming preprocessor

# SYNOPSIS

kaiseki [\<file\> ...]

kaiseki (--help | -h)

kaiseki --version

# DESCRIPTION

**kaiseki** aims to be a simple-to-use, humble preprocessor. **kaiseki**
believes in the worth of literate programming, of writing programs for
people to read first. To that end, it follows these principles:

**Literate programming should be plaintext**

Too many literate programming systems get hung up on typesetting code into
a beautiful book for everyone to read. But you can't open up a PDF in an
editor and start hacking on the code, and that's the best way for someone
to learn a codebase. **kaiseki** only takes plaintext as input, and only
spits out plaintext as output. **kaiseki** source files are just normal
source code, same as the ones you edit every day in your editor. And that
means you can learn the code the same way you always do: by poking it and
seeing what it does.

**Literate programming should be layered**

Many literate programming systems aim to be monolithic. The output is a book,
or a research paper. All the code goes into a single file, and there's no
way to figure out where anything is. You have to read the whole thing in order
to understand the structure. Flouting the wisdom of using the filesystem
as a hierarchy, of naming files so that you know what their contents are,
does not lead to good code. 

There's no good way to work *across* files. For many literate programming
systems, breaking things across files is a nightmare. The structure becomes
jumbled and confused; instead of elucidating the structure of the program,
it gets hidden underneath a mess of included files and tags. Wasn't this
the problem literate programming was meant to solve?

**kaiseki** instead encourages building the program in layers. Instead of
defining independent blocks which get cobbled together in the *real*
program, **kaiseki** *starts* from a working program and adds things into
it. First, a working, compiling program with nothing in it. Each layer
then *inserts* things into the previous to add on to it, each one continuing
to produce something that works. This better facilitates learning the code;
someone coming into the codebase can go upwards, layer by layer, gradually
figuring out each piece before moving on to the next one, instead of
being dumped headfirst into the codebase.

**Literate programming should be simple**

You shouldn't have to learn LaTeX or memorize a dozen arcane commands merely
to write a program that's readable. **kaiseki** has only **4** commands,
easily understood: **insert**, **label**, **before**, and **after**. That's
it!

# SYNTAX

**kaiseki** believes in the longevity of plaintext. As such, input to **kaiseki**
is simply normal source code files, with one catch: any line containing
"**anchors**" is interpreted as a special directive to **kaiseki**.

An **anchor** looks like: `##[<command>[(<arg>)]]`

Take the following C code:

```
/* ##[label(Includes)] */
/* ##[label(Forward Declarations)] */

int main(int argc, char* argv[]) {
    /* ##[label(Initialization)] */

    return 0;
}

/* ##[after(Includes)] */

#include <stdio.h>
#include <stdlib.h>

/* ##[before(Initialization)] */

char* buf = malloc(sizeof(char) * 1024);

/* ##[insert] */
/* Now we're back to adding code after `main()`. */

void foo() {
    printf("Hello!");
}

```

The output would (approximately) be:

```
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char* argv[]) {
    char* buf = malloc(sizeof(char) * 1024);

    return 0;
}

void foo() {
    printf("Hello!");
}
```

# COMMANDS

There are **4** commands that can be used in **anchors**.

**insert**

Don't do a rearrangement on the following block of lines. Simply
place them as-is at the end of the output.

**label** <*arg*>

Create a new insertion point wherever lines are currently being inserted.
Can be used inside the "scope" of a `##[before(<label>)]` or a `##[after(<label>)]`.

An intuitive way to think about it is that inserted lines will "expand" outward from
the position of the label in the text.

**before** <*arg*>

Insert the following block of lines *before* the given label.

If multiple blocks get inserted before a given label, the *first*
block seen and processed will be the *last* to appear in the output.

**after** <*arg*>

Insert the following block of lines *after* the given label.

If multiple blocks get inserted after a given label, the *first*
block seen and processed will be the *first* to appear in the output.

# SEE ALSO

**noweb**(1)
Another literate programming tool. Suffers from many of the problems
discussed above with literate programming.
