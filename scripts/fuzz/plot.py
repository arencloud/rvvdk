#!/usr/bin/env python3
"""Render deterministic fuzz feedback/rate/resource plots from retained logs."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import statistics

import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('report', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    source = args.report/'campaigns/runs.json'
    document = json.loads(source.read_text())
    runs = document['runs']
    assert len(runs) == 4*document['repeats']
    rows = []
    for name in ['descriptor', 'header', 'metadata', 'chain']:
        selected = [r for r in runs if r['target'] == name]
        for r in selected:
            text = (args.report/'campaigns'/(r['tag']+'.txt')).read_text()
            stats = {k: int(v) for k,v in re.findall(r'stat::(\w+):\s+(\d+)', text)}
            feedback = [dict(executions=int(n), event=event, coverage=int(cov), features=int(ft)) for n,event,cov,ft in re.findall(r'#(\d+)\s+(INITED|NEW|REDUCE|pulse|DONE)\s+cov:\s*(\d+)\s+ft:\s*(\d+)', text)]
            assert r['exit_code'] == 0 and stats == r['stats'] and feedback == r['feedback']
            assert feedback[0]['event'] == 'INITED' and feedback[-1]['event'] == 'DONE'
            assert feedback[-1]['executions'] == stats['number_of_executed_units']
            assert stats['peak_rss_mb'] <= 1024
        rows.append(dict(target=name, executions=[r['stats']['number_of_executed_units'] for r in selected],
                         rates=[r['stats']['average_exec_per_sec'] for r in selected],
                         rss_mb=[r['stats']['peak_rss_mb'] for r in selected],
                         initial_coverage=[r['feedback'][0]['coverage'] for r in selected],
                         final_coverage=[r['feedback'][-1]['coverage'] for r in selected],
                         initial_features=[r['feedback'][0]['features'] for r in selected],
                         final_features=[r['feedback'][-1]['features'] for r in selected]))
    output = args.output or args.report/'plots'; output.mkdir(parents=True, exist_ok=True)
    plt.rcParams.update({'svg.hashsalt': 'rvvdk-r59', 'font.family': 'DejaVu Sans', 'font.size': 11})
    fig, axes = plt.subplots(2,2,figsize=(13,9))
    names = [r['target'].capitalize() for r in rows]
    for ax,key,title,ylabel in zip(axes.flat,['rates','rss_mb','final_coverage','final_features'],
                                  ['Sanitized execution rate','Process memory high-water mark','Final feedback counters','Final feedback features'],
                                  ['Executions / second (log scale)','Peak RSS (MiB)','Covered counters · whole target','Features · whole target']):
        medians=[statistics.median(r[key]) for r in rows]
        ax.bar(names,medians,color='#2878a5',width=0.65,alpha=0.85)
        for index,row in enumerate(rows):
            offsets=[(n-(len(row[key])-1)/2)*0.13 for n in range(len(row[key]))]
            ax.scatter([index+v for v in offsets],row[key],facecolors='white',edgecolors='#152238',s=35,zorder=3)
        ax.set_title(title,pad=14);ax.set_ylabel(ylabel);ax.set_axisbelow(True);ax.grid(axis='y',alpha=0.2)
        if key=='rates':ax.set_yscale('log')
        for spine in ['top','right']:ax.spines[spine].set_visible(False)
    fig.suptitle('R5.9 · Bounded admission fuzz qualification',fontsize=20,fontweight='bold',y=0.98)
    fig.text(0.06,0.025,f"{document['repeats']} independent runs / target · {document['seconds']} s target · fixed seeds · ASan + overflow checks\nBars: median · dots: every run · CPUs 2–6 / shared host · no prior matched fuzz baseline\nFeedback includes harness code; it is not a source coverage percentage. Rates are not storage throughput.",fontsize=10,color='#46566b')
    fig.tight_layout(rect=(0.02,0.11,0.99,0.94))
    for extension in ['svg','png']:
        metadata={'Creator':'rvvdk fuzz plots','Date':None} if extension=='svg' else {'Software':'rvvdk fuzz plots'}
        fig.savefig(output/('qualification.'+extension),dpi=160,metadata=metadata)
    plt.close(fig)
    (output/'computed.json').write_text(json.dumps(rows,indent=2)+'\n')
    manifest=dict(generator_sha256=sha(Path(__file__)),matplotlib=matplotlib.__version__,input_sha256={str(source.relative_to(args.report)):sha(source)},
                  output_sha256={name:sha(output/name) for name in ['qualification.svg','qualification.png','computed.json']})
    (output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print('Verified',len(runs),'campaigns; plots saved to',output)


if __name__=='__main__':
    main()
