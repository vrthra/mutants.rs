#!/usr/bin/env python3
import warnings
warnings.simplefilter(action='ignore', category=FutureWarning)
import sys
import os

import ggplot as gg
import pandas as pd


def plot(mydata, opts, lim):
    # number of mutants killed by exactly 0 tests
    nd = mydata[mydata.exactly == 0]
    title = opts['title'] + (' ND=%d/%d (Mu: %3.1f%%)' % (len(nd), len(mydata), (1 - len(nd) / len(mydata))*100.0 ))
    p = gg.ggplot(gg.aes(x=opts['x'], y=opts['y']), data=mydata) + gg.geom_point() +\
            gg.xlab(opts['x']) + gg.ylab(opts['y']) + gg.ggtitle(title)  + \
       gg.xlim(0,lim)

    p.save(opts['file'])

def do_statistics(data, fn):
    plot(data, {'x':'ntests', 'y':'atleast', 'title':fn.replace('_', ' '), 'file':fn + '-atleast.png'}, 1000)
    plot(data, {'x':'ntests', 'y':'exactly', 'title':fn.replace('_', ' '), 'file':fn + '-exact.png'}, 1000)
    plot(data, {'x':'ntests', 'y':'atmost', 'title':fn.replace('_', ' '), 'file':fn + '-atmost.png'}, 1000)
    print('done', file=sys.stderr)

def main(args):
    fn = args[0]
    do_statistics(pd.read_csv(fn), fn)

main(sys.argv[1:])
