000: 687e  JPL 07e
002: 50    XD 0
003: 12    INK
004: 3f04  DIS 0x4
006: 3d01  JPA 0x1
008: 54    XYD 4
009: 1b    XX
00a: 51    XD 1
00b: 59    LXI 1
00c: 83    LYI 0x3
00d: 25    TSTI
00e: d0    JMP 010
00f: cd    JMP 00d
010: 25    TSTI
011: d0    JMP 010
012: 00    NOP
013: 12    INK
014: b0    CI 0x0
015: dc    JMP 01c
016: 57    XYD 7
017: 8a    LYI 0xa
018: 9a    LI 0xa
019: 1d    ST
01a: 57    XYD 7
01b: 90    LI 0x0
01c: 1a    STDC
01d: af    CYI 0xf
01e: cd    JMP 00d
01f: 51    XD 1
020: 1b    XX
021: 54    XYD 4
022: 56    XYD 6
023: 84    LYI 0x4
024: 56    XYD 6
025: 50    XD 0
026: 3e04  EN 0x4
028: 3c    RTI
029: 00    NOP
02a: 00    NOP
02b: 00    NOP
02c: 00    NOP
02d: 00    NOP
02e: 00    NOP
02f: 00    NOP
030: 00    NOP
031: 00    NOP
032: 00    NOP
033: 00    NOP
034: 00    NOP
035: 00    NOP
036: 00    NOP
037: 00    NOP
038: 00    NOP
039: 00    NOP
03a: 00    NOP
03b: 00    NOP
03c: 00    NOP
03d: 00    NOP
03e: 00    NOP
03f: 00    NOP
040: 6822  JPL 022
042: 00    NOP
043: 00    NOP
044: 6808  JPL 008
046: 00    NOP
047: 00    NOP
048: 90    LI 0x0
049: 53    XD 3
04a: 6822  JPL 022
04c: 91    LI 0x1
04d: 52    XD 2
04e: 6822  JPL 022
050: 90    LI 0x0
051: 52    XD 2
052: 6822  JPL 022
054: 91    LI 0x1
055: 53    XD 3
056: 6822  JPL 022
058: 6822  JPL 022
05a: 00    NOP
05b: 00    NOP
05c: 6822  JPL 022
05e: 00    NOP
05f: 00    NOP
060: 6a4f  JPL 24f
062: 00    NOP
063: 00    NOP
064: 6a4f  JPL 24f
066: 00    NOP
067: 00    NOP
068: 6a4f  JPL 24f
06a: 00    NOP
06b: 00    NOP
06c: 6a4f  JPL 24f
06e: 00    NOP
06f: 00    NOP
070: 6a4f  JPL 24f
072: 00    NOP
073: 00    NOP
074: 6a4f  JPL 24f
076: 00    NOP
077: 00    NOP
078: 6a4f  JPL 24f
07a: 00    NOP
07b: 00    NOP
07c: 6a4f  JPL 24f
07e: 58    LXI 0
07f: 80    LYI 0x0
080: 90    LI 0x0
081: 1b    XX
082: 0a    STIC
083: c2    JMP 082
084: 1b    XX
085: 71    AI 0x1
086: b8    CI 0x8
087: c1    JMP 081
088: 59    LXI 1
089: 8f    LYI 0xf
08a: 9c    LI 0xc
08b: 1d    ST
08c: 5a    LXI 2
08d: 8a    LYI 0xa
08e: 9f    LI 0xf
08f: 0a    STIC
090: 1d    ST
091: 90    LI 0x0
092: 07    TAS
093: 91    LI 0x1
094: 53    XD 3
095: 3e64  EN 0x64
097: 59    LXI 1
098: 88    LYI 0x8
099: 90    LI 0x0
09a: 1d    ST
09b: 58    LXI 0
09c: 89    LYI 0x9
09d: 16    TTLA
09e: 1d    ST
09f: 83    LYI 0x3
0a0: 0d    L
0a1: b1    CI 0x1
0a2: 68be  JPL 0be
0a4: 5a    LXI 2
0a5: 82    LYI 0x2
0a6: 13    IN
0a7: 84    LYI 0x4
0a8: 1a    STDC
0a9: 13    IN
0aa: 85    LYI 0x5
0ab: 1d    ST
0ac: 81    LYI 0x1
0ad: 13    IN
0ae: 87    LYI 0x7
0af: 1d    ST
0b0: 80    LYI 0x0
0b1: 13    IN
0b2: 86    LYI 0x6
0b3: 1d    ST
0b4: 9f    LI 0xf
0b5: 88    LYI 0x8
0b6: 0a    STIC
0b7: 0a    STIC
0b8: 0a    STIC
0b9: 0a    STIC
0ba: 90    LI 0x0
0bb: 07    TAS
0bc: 697e  JPL 17e
0be: 83    LYI 0x3
0bf: 13    IN
0c0: 4f    TBA 3
0c1: 6a39  JPL 239
0c3: 59    LXI 1
0c4: 8a    LYI 0xa
0c5: 0b    X
0c6: ba    CI 0xa
0c7: ca    JMP 0ca
0c8: 0b    X
0c9: e7    JMP 0e7
0ca: 0b    X
0cb: 85    LYI 0x5
0cc: 0b    X
0cd: 08    ICY
0ce: 0b    X
0cf: 08    ICY
0d0: 0b    X
0d1: 0f    AND
0d2: 2d    NEG
0d3: 7f    AI 0xf
0d4: 18    DCY
0d5: 1f    OR
0d6: 18    DCY
0d7: 1f    OR
0d8: 18    DCY
0d9: 1d    ST
0da: 4c    TBA 0
0db: 6187  CALL 187
0dd: 84    LYI 0x4
0de: 0d    L
0df: 4d    TBA 1
0e0: 61a4  CALL 1a4
0e2: 84    LYI 0x4
0e3: 0d    L
0e4: 4e    TBA 2
0e5: 61c6  CALL 1c6
0e7: 6265  CALL 265
0e9: 82    LYI 0x2
0ea: 13    IN
0eb: 5a    LXI 2
0ec: 0b    X
0ed: 18    DCY
0ee: 0b    X
0ef: 18    DCY
0f0: 0b    X
0f1: 0f    AND
0f2: 2d    NEG
0f3: 7f    AI 0xf
0f4: 08    ICY
0f5: 1f    OR
0f6: 08    ICY
0f7: 1f    OR
0f8: 08    ICY
0f9: 1d    ST
0fa: 58    LXI 0
0fb: 83    LYI 0x3
0fc: 0d    L
0fd: b0    CI 0x0
0fe: 69f9  JPL 1f9
100: 59    LXI 1
101: 88    LYI 0x8
102: 0d    L
103: b0    CI 0x0
104: 6920  JPL 120
106: 8a    LYI 0xa
107: 0d    L
108: b0    CI 0x0
109: cf    JMP 10f
10a: 18    DCY
10b: 0d    L
10c: b0    CI 0x0
10d: d9    JMP 119
10e: e0    JMP 120
10f: 5a    LXI 2
110: 83    LYI 0x3
111: 0d    L
112: 4e    TBA 2
113: 69d9  JPL 1d9
115: 4f    TBA 3
116: 69d7  JPL 1d7
118: e0    JMP 120
119: b1    CI 0x1
11a: cf    JMP 10f
11b: 5a    LXI 2
11c: 83    LYI 0x3
11d: 3a    TBIT 2
11e: 69d9  JPL 1d9
120: 59    LXI 1
121: 8f    LYI 0xf
122: 9c    LI 0xc
123: 0f    AND
124: 1d    ST
125: 8b    LYI 0xb
126: 0d    L
127: b0    CI 0x0
128: eb    JMP 12b
129: 693d  JPL 13d
12b: 16    TTLA
12c: 8f    LYI 0xf
12d: b4    CI 0x4
12e: 6939  JPL 139
130: 3b    TBIT 3
131: f4    JMP 134
132: 693d  JPL 13d
134: 33    SBIT 3
135: 8b    LYI 0xb
136: 19    DCM
137: 693d  JPL 13d
139: b0    CI 0x0
13a: 693d  JPL 13d
13c: 37    RBIT 3
13d: 8c    LYI 0xc
13e: 0d    L
13f: b0    CI 0x0
140: c3    JMP 143
141: 6952  JPL 152
143: 16    TTLA
144: 8f    LYI 0xf
145: bc    CI 0xc
146: 694f  JPL 14f
148: 3a    TBIT 2
149: cb    JMP 14b
14a: d2    JMP 152
14b: 32    SBIT 2
14c: 8c    LYI 0xc
14d: 19    DCM
14e: d2    JMP 152
14f: b8    CI 0x8
150: d2    JMP 152
151: 36    RBIT 2
152: 58    LXI 0
153: 83    LYI 0x3
154: 0d    L
155: b0    CI 0x0
156: ef    JMP 16f
157: 8a    LYI 0xa
158: 59    LXI 1
159: 0d    L
15a: b0    CI 0x0
15b: e1    JMP 161
15c: 18    DCY
15d: 0d    L
15e: b1    CI 0x1
15f: e1    JMP 161
160: e9    JMP 169
161: b0    CI 0x0
162: e4    JMP 164
163: ef    JMP 16f
164: 15    TTHA
165: 4c    TBA 0
166: e9    JMP 169
167: 8f    LYI 0xf
168: 09    ICM
169: 15    TTHA
16a: 4c    TBA 0
16b: ef    JMP 16f
16c: 8f    LYI 0xf
16d: 09    ICM
16e: 09    ICM
16f: 8f    LYI 0xf
170: 59    LXI 1
171: 0d    L
172: 02    OUTP
173: 59    LXI 1
174: 8a    LYI 0xa
175: 0d    L
176: b0    CI 0x0
177: 697c  JPL 17c
179: 9f    LI 0xf
17a: 07    TAS
17b: fe    JMP 17e
17c: 90    LI 0x0
17d: 07    TAS
17e: 58    LXI 0
17f: 89    LYI 0x9
180: 3e60  EN 0x60
182: 16    TTLA
183: 2e    C
184: 6897  JPL 097
186: c2    JMP 182
187: 83    LYI 0x3
188: 0d    L
189: 8d    LYI 0xd
18a: 09    ICM
18b: 2e    C
18c: df    JMP 19f
18d: 90    LI 0x0
18e: 1d    ST
18f: 82    LYI 0x2
190: 0d    L
191: 88    LYI 0x8
192: 0a    STIC
193: 23    RSTC
194: 0e    ADC
195: 10    DAA
196: 0a    STIC
197: 90    LI 0x0
198: 0e    ADC
199: 1d    ST
19a: ba    CI 0xa
19b: df    JMP 19f
19c: 99    LI 0x9
19d: 1a    STDC
19e: 1d    ST
19f: 8b    LYI 0xb
1a0: 09    ICM
1a1: e3    JMP 1a3
1a2: 19    DCM
1a3: 2c    RTS
1a4: 81    LYI 0x1
1a5: 0d    L
1a6: 8e    LYI 0xe
1a7: 09    ICM
1a8: 2e    C
1a9: 69c1  JPL 1c1
1ab: 90    LI 0x0
1ac: 1d    ST
1ad: 80    LYI 0x0
1ae: 0d    L
1af: 88    LYI 0x8
1b0: 0b    X
1b1: 1f    OR
1b2: 0b    X
1b3: 08    ICY
1b4: 23    RSTC
1b5: 0e    ADC
1b6: 10    DAA
1b7: 0a    STIC
1b8: 90    LI 0x0
1b9: 0e    ADC
1ba: 1d    ST
1bb: ba    CI 0xa
1bc: 69c1  JPL 1c1
1be: 99    LI 0x9
1bf: 1a    STDC
1c0: 1d    ST
1c1: 8c    LYI 0xc
1c2: 09    ICM
1c3: c5    JMP 1c5
1c4: 19    DCM
1c5: 2c    RTS
1c6: 91    LI 0x1
1c7: 88    LYI 0x8
1c8: 1d    ST
1c9: 08    ICY
1ca: 23    RSTC
1cb: 0e    ADC
1cc: 10    DAA
1cd: 0a    STIC
1ce: 90    LI 0x0
1cf: 0e    ADC
1d0: 1d    ST
1d1: ba    CI 0xa
1d2: d6    JMP 1d6
1d3: 99    LI 0x9
1d4: 1a    STDC
1d5: 1d    ST
1d6: 2c    RTS
1d7: 92    LI 0x2
1d8: da    JMP 1da
1d9: 91    LI 0x1
1da: 89    LYI 0x9
1db: 59    LXI 1
1dc: 23    RSTC
1dd: 1e    SBC
1de: 11    DAS
1df: 0a    STIC
1e0: 90    LI 0x0
1e1: 1e    SBC
1e2: 1d    ST
1e3: 6265  CALL 265
1e5: 92    LI 0x2
1e6: 58    LXI 0
1e7: 83    LYI 0x3
1e8: 1d    ST
1e9: 59    LXI 1
1ea: 8d    LYI 0xd
1eb: 90    LI 0x0
1ec: 0a    STIC
1ed: 1d    ST
1ee: 83    LYI 0x3
1ef: 0d    L
1f0: b0    CI 0x0
1f1: 6920  JPL 120
1f3: 89    LYI 0x9
1f4: 0a    STIC
1f5: 9a    LI 0xa
1f6: 1d    ST
1f7: 6920  JPL 120
1f9: 5a    LXI 2
1fa: 83    LYI 0x3
1fb: 91    LI 0x1
1fc: 0f    AND
1fd: 87    LYI 0x7
1fe: 1d    ST
1ff: 82    LYI 0x2
200: 91    LI 0x1
201: 0f    AND
202: 23    RSTC
203: 0c    ROL
204: 87    LYI 0x7
205: 1f    OR
206: 1d    ST
207: 58    LXI 0
208: 82    LYI 0x2
209: 0d    L
20a: 5a    LXI 2
20b: b0    CI 0x0
20c: d4    JMP 214
20d: 80    LYI 0x0
20e: 13    IN
20f: 6272  CALL 272
211: 86    LYI 0x6
212: 1d    ST
213: d8    JMP 218
214: 80    LYI 0x0
215: 13    IN
216: 86    LYI 0x6
217: 1d    ST
218: 5a    LXI 2
219: 83    LYI 0x3
21a: 92    LI 0x2
21b: 0f    AND
21c: 23    RSTC
21d: 1c    ROR
21e: 89    LYI 0x9
21f: 1d    ST
220: 82    LYI 0x2
221: 92    LI 0x2
222: 0f    AND
223: 89    LYI 0x9
224: 1f    OR
225: 1d    ST
226: 58    LXI 0
227: 82    LYI 0x2
228: 0d    L
229: 5a    LXI 2
22a: 81    LYI 0x1
22b: b0    CI 0x0
22c: f4    JMP 234
22d: 13    IN
22e: 6272  CALL 272
230: 88    LYI 0x8
231: 1d    ST
232: 6920  JPL 120
234: 13    IN
235: 88    LYI 0x8
236: 1d    ST
237: 6920  JPL 120
239: 59    LXI 1
23a: 89    LYI 0x9
23b: 90    LI 0x0
23c: 07    TAS
23d: 0a    STIC
23e: 1d    ST
23f: 8d    LYI 0xd
240: 0a    STIC
241: 1d    ST
242: 5a    LXI 2
243: 84    LYI 0x4
244: 9b    LI 0xb
245: 0a    STIC
246: 0a    STIC
247: 58    LXI 0
248: 83    LYI 0x3
249: 91    LI 0x1
24a: 1d    ST
24b: 9f    LI 0xf
24c: 02    OUTP
24d: 697e  JPL 17e
24f: 1b    XX
250: 51    XD 1
251: 56    XYD 6
252: 5a    LXI 2
253: 0d    L
254: 23    RSTC
255: 01    OUTO
256: 08    ICY
257: 0d    L
258: 21    SETC
259: 01    OUTO
25a: 08    ICY
25b: ac    CYI 0xc
25c: de    JMP 25e
25d: 84    LYI 0x4
25e: 56    XYD 6
25f: 51    XD 1
260: 1b    XX
261: 50    XD 0
262: 3e04  EN 0x4
264: 3c    RTI
265: 59    LXI 1
266: 89    LYI 0x9
267: 0d    L
268: 84    LYI 0x4
269: 5a    LXI 2
26a: 1d    ST
26b: 59    LXI 1
26c: 8a    LYI 0xa
26d: 0d    L
26e: 5a    LXI 2
26f: 85    LYI 0x5
270: 1d    ST
271: 2c    RTS
272: 3d0a  JPA 0xa
274: 00    NOP
275: 00    NOP
276: 00    NOP
277: 00    NOP
278: 00    NOP
279: 00    NOP
27a: 00    NOP
27b: 00    NOP
27c: 00    NOP
27d: 00    NOP
27e: 00    NOP
27f: 00    NOP
280: 9f    LI 0xf
281: 2c    RTS
282: 00    NOP
283: 00    NOP
284: 9e    LI 0xe
285: 2c    RTS
286: 00    NOP
287: 00    NOP
288: 9d    LI 0xd
289: 2c    RTS
28a: 00    NOP
28b: 00    NOP
28c: 95    LI 0x5
28d: 2c    RTS
28e: 00    NOP
28f: 00    NOP
290: 9c    LI 0xc
291: 2c    RTS
292: 00    NOP
293: 00    NOP
294: 99    LI 0x9
295: 2c    RTS
296: 00    NOP
297: 00    NOP
298: 97    LI 0x7
299: 2c    RTS
29a: 00    NOP
29b: 00    NOP
29c: 96    LI 0x6
29d: 2c    RTS
29e: 00    NOP
29f: 00    NOP
2a0: 9b    LI 0xb
2a1: 2c    RTS
2a2: 00    NOP
2a3: 00    NOP
2a4: 93    LI 0x3
2a5: 2c    RTS
2a6: 00    NOP
2a7: 00    NOP
2a8: 9a    LI 0xa
2a9: 2c    RTS
2aa: 00    NOP
2ab: 00    NOP
2ac: 94    LI 0x4
2ad: 2c    RTS
2ae: 00    NOP
2af: 00    NOP
2b0: 91    LI 0x1
2b1: 2c    RTS
2b2: 00    NOP
2b3: 00    NOP
2b4: 92    LI 0x2
2b5: 2c    RTS
2b6: 00    NOP
2b7: 00    NOP
2b8: 90    LI 0x0
2b9: 2c    RTS
2ba: 00    NOP
2bb: 00    NOP
2bc: 98    LI 0x8
2bd: 2c    RTS
2be: 00    NOP
2bf: 00    NOP
2c0: 00    NOP
2c1: 00    NOP
2c2: 00    NOP
2c3: 00    NOP
2c4: 00    NOP
2c5: 00    NOP
2c6: 00    NOP
2c7: 00    NOP
2c8: 00    NOP
2c9: 00    NOP
2ca: 00    NOP
2cb: 00    NOP
2cc: 00    NOP
2cd: 00    NOP
2ce: 00    NOP
2cf: 00    NOP
2d0: 00    NOP
2d1: 00    NOP
2d2: 00    NOP
2d3: 00    NOP
2d4: 00    NOP
2d5: 00    NOP
2d6: 00    NOP
2d7: 00    NOP
2d8: 00    NOP
2d9: 00    NOP
2da: 00    NOP
2db: 00    NOP
2dc: 00    NOP
2dd: 00    NOP
2de: 00    NOP
2df: 00    NOP
2e0: 00    NOP
2e1: 00    NOP
2e2: 00    NOP
2e3: 00    NOP
2e4: 00    NOP
2e5: 00    NOP
2e6: 00    NOP
2e7: 00    NOP
2e8: 00    NOP
2e9: 00    NOP
2ea: 00    NOP
2eb: 00    NOP
2ec: 00    NOP
2ed: 00    NOP
2ee: 00    NOP
2ef: 00    NOP
2f0: 00    NOP
2f1: 00    NOP
2f2: 00    NOP
2f3: 00    NOP
2f4: 00    NOP
2f5: 00    NOP
2f6: 00    NOP
2f7: 00    NOP
2f8: 00    NOP
2f9: 00    NOP
2fa: 00    NOP
2fb: 00    NOP
2fc: 00    NOP
2fd: 00    NOP
2fe: 00    NOP
2ff: 00    NOP
300: 00    NOP
301: 00    NOP
302: 00    NOP
303: 00    NOP
304: 00    NOP
305: 00    NOP
306: 00    NOP
307: 00    NOP
308: 00    NOP
309: 00    NOP
30a: 00    NOP
30b: 00    NOP
30c: 00    NOP
30d: 00    NOP
30e: 00    NOP
30f: 00    NOP
310: 00    NOP
311: 00    NOP
312: 00    NOP
313: 00    NOP
314: 00    NOP
315: 00    NOP
316: 00    NOP
317: 00    NOP
318: 00    NOP
319: 00    NOP
31a: 00    NOP
31b: 00    NOP
31c: 00    NOP
31d: 00    NOP
31e: 00    NOP
31f: 00    NOP
320: 00    NOP
321: 00    NOP
322: 00    NOP
323: 00    NOP
324: 00    NOP
325: 00    NOP
326: 00    NOP
327: 00    NOP
328: 00    NOP
329: 00    NOP
32a: 00    NOP
32b: 00    NOP
32c: 00    NOP
32d: 00    NOP
32e: 00    NOP
32f: 00    NOP
330: 00    NOP
331: 00    NOP
332: 00    NOP
333: 00    NOP
334: 00    NOP
335: 00    NOP
336: 00    NOP
337: 00    NOP
338: 00    NOP
339: 00    NOP
33a: 00    NOP
33b: 00    NOP
33c: 00    NOP
33d: 00    NOP
33e: 00    NOP
33f: 00    NOP
340: 00    NOP
341: 00    NOP
342: 00    NOP
343: 00    NOP
344: 00    NOP
345: 00    NOP
346: 00    NOP
347: 00    NOP
348: 00    NOP
349: 00    NOP
34a: 00    NOP
34b: 00    NOP
34c: 00    NOP
34d: 00    NOP
34e: 00    NOP
34f: 00    NOP
350: 00    NOP
351: 00    NOP
352: 00    NOP
353: 00    NOP
354: 00    NOP
355: 00    NOP
356: 00    NOP
357: 00    NOP
358: 00    NOP
359: 00    NOP
35a: 00    NOP
35b: 00    NOP
35c: 00    NOP
35d: 00    NOP
35e: 00    NOP
35f: 00    NOP
360: 00    NOP
361: 00    NOP
362: 00    NOP
363: 00    NOP
364: 00    NOP
365: 00    NOP
366: 00    NOP
367: 00    NOP
368: 00    NOP
369: 00    NOP
36a: 00    NOP
36b: 00    NOP
36c: 00    NOP
36d: 00    NOP
36e: 00    NOP
36f: 00    NOP
370: 00    NOP
371: 00    NOP
372: 00    NOP
373: 00    NOP
374: 00    NOP
375: 00    NOP
376: 00    NOP
377: 00    NOP
378: 00    NOP
379: 00    NOP
37a: 00    NOP
37b: 00    NOP
37c: 00    NOP
37d: 00    NOP
37e: 00    NOP
37f: 00    NOP
380: 00    NOP
381: 00    NOP
382: 00    NOP
383: 00    NOP
384: 00    NOP
385: 00    NOP
386: 00    NOP
387: 00    NOP
388: 00    NOP
389: 00    NOP
38a: 00    NOP
38b: 00    NOP
38c: 00    NOP
38d: 00    NOP
38e: 00    NOP
38f: 00    NOP
390: 00    NOP
391: 00    NOP
392: 00    NOP
393: 00    NOP
394: 00    NOP
395: 00    NOP
396: 00    NOP
397: 00    NOP
398: 00    NOP
399: 00    NOP
39a: 00    NOP
39b: 00    NOP
39c: 00    NOP
39d: 00    NOP
39e: 00    NOP
39f: 00    NOP
3a0: 00    NOP
3a1: 00    NOP
3a2: 00    NOP
3a3: 00    NOP
3a4: 00    NOP
3a5: 00    NOP
3a6: 00    NOP
3a7: 00    NOP
3a8: 00    NOP
3a9: 00    NOP
3aa: 00    NOP
3ab: 00    NOP
3ac: 00    NOP
3ad: 00    NOP
3ae: 00    NOP
3af: 00    NOP
3b0: 00    NOP
3b1: 00    NOP
3b2: 00    NOP
3b3: 00    NOP
3b4: 00    NOP
3b5: 00    NOP
3b6: 00    NOP
3b7: 00    NOP
3b8: 00    NOP
3b9: 00    NOP
3ba: 00    NOP
3bb: 00    NOP
3bc: 00    NOP
3bd: 00    NOP
3be: 00    NOP
3bf: 00    NOP
3c0: 00    NOP
3c1: 00    NOP
3c2: 00    NOP
3c3: 00    NOP
3c4: 00    NOP
3c5: 00    NOP
3c6: 00    NOP
3c7: 00    NOP
3c8: 00    NOP
3c9: 00    NOP
3ca: 00    NOP
3cb: 00    NOP
3cc: 00    NOP
3cd: 00    NOP
3ce: 00    NOP
3cf: 00    NOP
3d0: 00    NOP
3d1: 00    NOP
3d2: 00    NOP
3d3: 00    NOP
3d4: 00    NOP
3d5: 00    NOP
3d6: 00    NOP
3d7: 00    NOP
3d8: 00    NOP
3d9: 00    NOP
3da: 00    NOP
3db: 00    NOP
3dc: 00    NOP
3dd: 00    NOP
3de: 00    NOP
3df: 00    NOP
3e0: 00    NOP
3e1: 00    NOP
3e2: 00    NOP
3e3: 00    NOP
3e4: 00    NOP
3e5: 00    NOP
3e6: 00    NOP
3e7: 00    NOP
3e8: 00    NOP
3e9: 00    NOP
3ea: 00    NOP
3eb: 00    NOP
3ec: 00    NOP
3ed: 00    NOP
3ee: 00    NOP
3ef: 00    NOP
3f0: 00    NOP
3f1: 00    NOP
3f2: 00    NOP
3f3: 00    NOP
3f4: 00    NOP
3f5: 00    NOP
3f6: 00    NOP
3f7: 00    NOP
3f8: 00    NOP
3f9: 00    NOP
3fa: 00    NOP
3fb: 00    NOP
3fc: 00    NOP
3fd: 00    NOP
3fe: 00    NOP
3ff: 00    NOP
