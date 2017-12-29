#!/usr/bin/env python3
import warnings
warnings.simplefilter(action='ignore', category=FutureWarning)
import sys
import os
import io

import ggplot as gg
import pandas as pd


def plot(mydata, opts):
    # number of mutants killed by exactly 0 tests
    nd = sum(mydata[mydata.ntests == 0].exactly)
    d = sum(mydata[mydata.ntests != 0].exactly)
    total = nd + d
    print("Not detected = ", nd, "/", total)
    title = opts['title'] + (' ND=%d/%d (Mu: %3.1f%%)' % (nd, total, (1 - nd/ total)*100.0 ))
    p = gg.ggplot(gg.aes(x=opts['x'], y=opts['y']), data=mydata) + gg.geom_point() +\
            gg.xlab(opts['x']) + gg.ylab(opts['y']) + gg.ggtitle(title)  #+ \
    #   gg.xlim(0,lim)

    p.save(opts['file'])

def sanitize(s):
    return s.replace('_', ' ').replace('kills.csv', '')

def plotit(data, fn):
    print('exact')
    plot(data, {'x':'ntests', 'y':'exactly', 'title':sanitize(fn), 'file':fn + '-exact.png'})
    print('atleast')
    plot(data, {'x':'ntests', 'y':'atleast', 'title':sanitize(fn), 'file':fn + '-atleast.png'})
    print('atmost')
    plot(data, {'x':'ntests', 'y':'atmost', 'title':sanitize(fn), 'file':fn + '-atmost.png'})
    print('done', file=sys.stderr)

def main(args):
    fn = args[0]
    v = open(fn).read()
    d =  pd.read_csv(io.StringIO(v.replace(' ', '')))
    plotit(d, fn)

main(sys.argv[1:])
