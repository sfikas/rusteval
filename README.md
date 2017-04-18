# Rusteval

[![Build Status](https://travis-ci.org/sfikas/rusteval.svg?branch=master)](https://travis-ci.org/sfikas/rusteval)

A tool used to evaluate the output of retrieval algorithms. Written in [Rust]. 

## Building

Install [Rust] with
```
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```
and build the release version with

```
git clone <rusteval repo name>
cd rusteval
cargo build --release
```

## Testing

Before testing, unzip the test result file found in the ```fixtures/``` folder with (unzipped this is >100Mb):
```
gunzip fixtures/G1_TRACK_I_Bentham.xml.gz
```

Run the test suite with

```
cargo test --release
```

or, for a more verbose output

```
cargo test --release -- --nocapture
```

## Running

After building and testing, run rusteval with
```
target/release/rusteval <relevance file> <result file>
```

The ```fixtures/``` folder contains some examples of relevance and result files (see below for an explanation of what these files are).
For example, in order to reproduce some of the results of the ICFHR'14 [keyword spotting competition], you can run
```
target/release/rusteval fixtures/TRACK_I_Bentham_ICFHR2014.RelevanceJudgements.xml fixtures/G1_TRACK_I_Bentham.xml
```
This should produce the results of evaluation of method 'G1' for the 'Bentham' track of the competition. Results show up for each of the selected queries, and averaged over all queries.
The last lines of the output should read something like
```
MEAN:  precAt5    precAt10   ap              
=======================================================================
       0.73813    0.60268    0.52402		
```
This output means that mean precision at 5 is 73.8%, mean precision at 10 is 60.2%, and mean average precision (MAP) is 52.4% for the submitted method.

## The retrieval paradigm, relevance and result files

The retrieval paradigm typically presupposes a finite set of queries, each associated with a finite set of matching tokens.

A retrieval algorithm returns an ordered list for each query, representing all tokens from best to worst match.

This information is necessary for evaluation.
Input to the tool is read from two distinct text files, the *relevance file* and the *result file*.

The *relevance file* tells us:
* What and how many are our queries
* With what matching tokens does each query *actually* match

The *result file* tells us:
* What is the ordered list of matching tokens, from best to worst match, for each query 

## Supported input file formats

### trec_eval format

This format has been originally introduced for use with the [trec_eval] evaluation software.

#### Relevance file

Relevance files follow the format
```
qid  0  docno  rel
```
for each text line.

The line above tells us that query with id ```qid``` matches with token ```docno```.
The degree that the query and each token match is encoded as the floating-point value ```rel```, taking
values in ```[0, 1]```. A perfect match has ```rel = 1```. 

Sample relevance file:
```
cv1 0 tok1 1
cv1 0 tok2 1
cv1 0 tok3 0
cv2 0 tok1 0
cv2 0 tok2 0
cv2 0 tok3 1
```

This tells us that query ```cv1``` matches with tokens ```tok1``` and ```tok2``` but not ```tok3```;
query ```cv2``` matches with token ```tok3``` only.

#### Results file

Result files follow the format
```
qid 0 docno rank sim run_id
```
for each text line.

```rank``` is an integer that is ignored but required by the format, and has to be in the range ```[0, 1000]``` according the documentation.
```sim``` is a floating-point value. Higher ```sim``` corresponds to a better match.
```run_id``` is also required but ignored.

According to the docs, the file has to be sorted according to ```qid```.

Sample result file:
```
cv1 0 April_d06-086-09 0 -0.960748 hws
cv1 0 April_d05-008-03 1 -1.307986 hws
cv1 0 April_p03-181-00 2 -1.372011 hws
cv1 0 April_d05-021-05 3 -1.394318 hws
cv1 0 April_e06-053-07 4 -1.404273 hws
cv1 0 April_g01-025-09 5 -1.447217 hws
cv1 0 April_g01-027-03 6 -1.453828 hws
cv1 0 April_p03-072-03 7 -1.556320 hws
cv1 0 April_g01-008-03 8 -1.584332 hws
cv1 0 April_n01-045-05 9 -1.682590 hws
```

This shows results for matches with query ```cv1```. The best match is ```April_d06-086-09```, 
the worst match is ```April_n01-045-05```.
Note again that is is the ```rank``` value that encodes the order of the matches, i.e. the penultimate floating-point number in each line.

### icfhr'14 keyword spotting format

This format is adapted to be used with [keyword spotting], a form of image retrieval where retrieved elemens are word images, typically cropped off a containing document image.
It has been used for the ICFHR'14 [keyword spotting competition].

Tokens are defined with an XML ```word``` tag, that must contain the following fields *in this particular order*:
* document
* x
* y
* width
* height
* Text (optional)
* Relevance (optional; default value = 1)

Note also that rusteval requires that each line must contain at most one XML tag.

#### Relevance file

Sample relevance file:
```
<?xml version="1.0" encoding="utf-8"?>
<GroundTruthRelevanceJudgements xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <GTRel queryid="query1">
    <word document="027_029_001" x="159" y="1775" width="184" height="89" Text="possess" Relevance="1" />
    <word document="027_029_001" x="860" y="1774" width="180" height="89" Relevance="1" />
  </GTRel>
  <GTRel queryid="query2">
    <word document="027_029_001" x="1490" y="1769" width="176" height="86" Relevance="1" />
    <word document="071_053_004" x="354" y="790" width="319" height="108" Text="possesst" Relevance="0.7" />
    <word document="027_029_001" x="1460" y="178" width="298" height="98" Relevance="0.6" />
  </GTRel>
</GroundTruthRelevanceJudgements>
```

#### Result file

The quality of the match is encoded by the order in which the token appears in the file.

Sample result file:
```
<?xml version="1.0" encoding="utf-8"?><RelevanceListings xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Rel queryid="query1">
    <word document="027_029_001" x="159" y="1775" width="184" height="89" />
    <word document="027_029_001" x="860" y="1774" width="180" height="89" />
    <word document="027_029_001" x="1490" y="1769" width="176" height="86" />
    <word document="027_029_001" x="1015" y="2182" width="189" height="87" />
    <word document="071_053_004" x="92" y="607" width="220" height="138" />
  </Rel>
  <Rel queryid="query2">
    <word document="027_029_001" x="1015" y="2182" width="189" height="87" />
    <word document="071_053_004" x="92" y="607" width="220" height="138" />
    <word document="027_029_001" x="159" y="1775" width="184" height="89" />
    <word document="027_029_001" x="860" y="1774" width="180" height="89" />  
    <word document="027_029_001" x="1490" y="1769" width="176" height="86" />    
  </Rel>
</RelevanceListings>
```

In this example, for ```query2``` the best match is ```document="027_029_001" x="1015" y="2182" width="189" height="87"```,
and the worst match is ```document="027_029_001" x="1490" y="1769" width="176" height="86"```.

## Metrics

### Precision at 5

Precision at 5 is defined as the ratio of the number of instances, among the k closest matches, that are correctly retrieved,
divided by k.
For Precision at 5, k equals to 5, *or the total number of possible matches if this number is less than 5* (the software provided with the [keyword spotting competition] of ICFHR 2014 also uses this convention).
Precision at 10 is defined in an analogous manner.

### Average Precision

Average precision is defined as the weighted average of 'Precisions at k' for all possible values of k.
The weight depends on k and equals to one if the k-th retrieved instance is a match. Otherwise it equals to zero.  

For more details, see
```
@ARTICLE{Giotis17,
    title={A survey of document image word spotting techniques},
    author={A. P. Giotis and G. Sfikas and B. Gatos and C. Nikou},
    journal={Pattern Recognition},
    year={2017},
    publisher={Elsevier}
}
```


[trec_eval]: <http://faculty.washington.edu/levow/courses/ling573_SPR2011/hw/trec_eval_desc.htm>
[keyword spotting]: <http://www.cs.uoi.gr/~sfikas/16SfikasRetsinasGatos_ZAH.pdf>
[keyword spotting competition]: <http://vc.ee.duth.gr/H-KWS2014/>
[Rust]: <https://www.rust-lang.org/>
