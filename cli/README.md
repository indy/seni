

running with logging levels:

$ RUST_LOG=trace ./target/debug/cli script.seni

--------------------------------------------------------------------------------

recording performance/flamegraph data:
(requires perf and flamegraph)

$ sudo perf record -g ./target/release/cli 1841-nib.seni
$ sudo perf script | /home/indy/repos/perl/FlameGraph/stackcollapse-perf.pl | /home/indy/repos/perl/FlameGraph/flamegraph.pl > flame.svg

--------------------------------------------------------------------------------

running benchmarks

$ cargo bench

then view ./target/criterion/report/index.html
(see https://bheisler.github.io/criterion.rs/book/user_guide/plots_and_graphs.html)


# installing tools
## gnuplot
    $ sudo apt install gnuplot

## perf
    get the perf tool by installing the linux-tools package. This is a virtual package so you need to specify the correct variation for your kernel.

    find out the kernel with:
    $ uname -r

    can then install the correct linux-tool variant:
    $ sudo apt install linux-tools-oem

## flamegraph
    $ cd repos/perl
    $ git clone https://github.com/brendangregg/FlameGraph.git
