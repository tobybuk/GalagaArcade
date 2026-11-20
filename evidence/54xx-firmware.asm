000: c6    JMP 006
001: c6    JMP 006
002: 52    XD 2
003: 12    INK
004: 3d06  JPA 0x6
006: 58    LXI 0
007: 80    LYI 0x0
008: 90    LI 0x0
009: 1b    XX
00a: 0a    STIC
00b: ca    JMP 00a
00c: 1b    XX
00d: 71    AI 0x1
00e: b8    CI 0x8
00f: c9    JMP 009
010: 3e04  EN 0x4
012: 59    LXI 1
013: 84    LYI 0x4
014: 0d    L
015: 1c    ROR
016: 18    DCY
017: 0d    L
018: 1c    ROR
019: 1a    STDC
01a: 0d    L
01b: 1c    ROR
01c: 1a    STDC
01d: 0d    L
01e: 1c    ROR
01f: 1a    STDC
020: 0d    L
021: 1c    ROR
022: 1d    ST
023: 1c    ROR
024: 83    LYI 0x3
025: 2f    EOR
026: 71    AI 0x1
027: 08    ICY
028: 0a    STIC
029: 0d    L
02a: 80    LYI 0x0
02b: 38    TBIT 0
02c: f8    JMP 038
02d: 23    RSTC
02e: 01    OUTO
02f: 88    LYI 0x8
030: 20    SETR
031: 08    ICY
032: 0d    L
033: 21    SETC
034: 0e    ADC
035: 1a    STDC
036: 6841  JPL 041
038: 88    LYI 0x8
039: 22    RSTR
03a: 23    RSTC
03b: 90    LI 0x0
03c: 01    OUTO
03d: 89    LYI 0x9
03e: 0d    L
03f: 0e    ADC
040: 1a    STDC
041: 0d    L
042: 7f    AI 0xf
043: 6890  JPL 090
045: 58    LXI 0
046: 19    DCM
047: 6890  JPL 090
049: 59    LXI 1
04a: 86    LYI 0x6
04b: 19    DCM
04c: 6890  JPL 090
04e: 08    ICY
04f: 19    DCM
050: 6890  JPL 090
052: 08    ICY
053: 19    DCM
054: 0d    L
055: b1    CI 0x1
056: d8    JMP 058
057: df    JMP 05f
058: b2    CI 0x2
059: ff    JMP 07f
05a: 90    LI 0x0
05b: 85    LYI 0x5
05c: 1d    ST
05d: 92    LI 0x2
05e: e6    JMP 066
05f: 5b    LXI 3
060: 81    LYI 0x1
061: 0d    L
062: 59    LXI 1
063: 85    LYI 0x5
064: 1d    ST
065: 91    LI 0x1
066: 23    RSTC
067: 0c    ROL
068: 04    TAY
069: 5b    LXI 3
06a: 0d    L
06b: b0    CI 0x0
06c: f5    JMP 075
06d: 50    XD 0
06e: 08    ICY
06f: 0d    L
070: b0    CI 0x0
071: f8    JMP 078
072: 88    LYI 0x8
073: 59    LXI 1
074: d3    JMP 053
075: 50    XD 0
076: 08    ICY
077: 0d    L
078: 59    LXI 1
079: 87    LYI 0x7
07a: 1a    STDC
07b: 50    XD 0
07c: 1d    ST
07d: 6890  JPL 090
07f: 85    LYI 0x5
080: 0d    L
081: 7f    AI 0xf
082: 6890  JPL 090
084: 0d    L
085: 23    RSTC
086: 1c    ROR
087: 23    RSTC
088: 0e    ADC
089: 1c    ROR
08a: 1d    ST
08b: 91    LI 0x1
08c: 88    LYI 0x8
08d: 1d    ST
08e: 6866  JPL 066
090: 89    LYI 0x9
091: 59    LXI 1
092: 93    LI 0x3
093: 0f    AND
094: b1    CI 0x1
095: d8    JMP 098
096: 8e    LYI 0xe
097: 09    ICM
098: 8a    LYI 0xa
099: 0d    L
09a: 8e    LYI 0xe
09b: 38    TBIT 0
09c: de    JMP 09e
09d: 90    LI 0x0
09e: 21    SETC
09f: 01    OUTO
0a0: 38    TBIT 0
0a1: e5    JMP 0a5
0a2: 89    LYI 0x9
0a3: 20    SETR
0a4: e7    JMP 0a7
0a5: 89    LYI 0x9
0a6: 22    RSTR
0a7: 8d    LYI 0xd
0a8: 0d    L
0a9: 7f    AI 0xf
0aa: 68fc  JPL 0fc
0ac: 58    LXI 0
0ad: 19    DCM
0ae: 68fc  JPL 0fc
0b0: 59    LXI 1
0b1: 8b    LYI 0xb
0b2: 19    DCM
0b3: 68fc  JPL 0fc
0b5: 08    ICY
0b6: 19    DCM
0b7: 68fc  JPL 0fc
0b9: 08    ICY
0ba: 19    DCM
0bb: 0d    L
0bc: b1    CI 0x1
0bd: 68c0  JPL 0c0
0bf: c9    JMP 0c9
0c0: b2    CI 0x2
0c1: 68ec  JPL 0ec
0c3: 90    LI 0x0
0c4: 8a    LYI 0xa
0c5: 1d    ST
0c6: 92    LI 0x2
0c7: 68d0  JPL 0d0
0c9: 5b    LXI 3
0ca: 89    LYI 0x9
0cb: 0d    L
0cc: 59    LXI 1
0cd: 8a    LYI 0xa
0ce: 1d    ST
0cf: 91    LI 0x1
0d0: 23    RSTC
0d1: 0c    ROL
0d2: 78    AI 0x8
0d3: 04    TAY
0d4: 5b    LXI 3
0d5: 0d    L
0d6: b0    CI 0x0
0d7: 68e3  JPL 0e3
0d9: 50    XD 0
0da: 08    ICY
0db: 0d    L
0dc: b0    CI 0x0
0dd: 68e6  JPL 0e6
0df: 59    LXI 1
0e0: 8d    LYI 0xd
0e1: 68ba  JPL 0ba
0e3: 50    XD 0
0e4: 08    ICY
0e5: 0d    L
0e6: 59    LXI 1
0e7: 8c    LYI 0xc
0e8: 1a    STDC
0e9: 50    XD 0
0ea: 1d    ST
0eb: fc    JMP 0fc
0ec: 8a    LYI 0xa
0ed: 0d    L
0ee: 7f    AI 0xf
0ef: fc    JMP 0fc
0f0: 0d    L
0f1: 23    RSTC
0f2: 1c    ROR
0f3: 23    RSTC
0f4: 0e    ADC
0f5: 1c    ROR
0f6: 1d    ST
0f7: 91    LI 0x1
0f8: 8d    LYI 0xd
0f9: 1d    ST
0fa: 68d0  JPL 0d0
0fc: 5a    LXI 2
0fd: 8f    LYI 0xf
0fe: 0d    L
0ff: 8b    LYI 0xb
100: 3b    TBIT 3
101: c3    JMP 103
102: 90    LI 0x0
103: 81    LYI 0x1
104: 03    OUT
105: 88    LYI 0x8
106: 0d    L
107: 8a    LYI 0xa
108: 23    RSTC
109: 0e    ADC
10a: 1d    ST
10b: 89    LYI 0x9
10c: 0d    L
10d: 8b    LYI 0xb
10e: 0e    ADC
10f: 1d    ST
110: 59    LXI 1
111: 80    LYI 0x0
112: 38    TBIT 0
113: e1    JMP 121
114: 5a    LXI 2
115: 86    LYI 0x6
116: 0d    L
117: 23    RSTC
118: 8a    LYI 0xa
119: 0e    ADC
11a: 1d    ST
11b: 87    LYI 0x7
11c: 0d    L
11d: 8b    LYI 0xb
11e: 0e    ADC
11f: 1d    ST
120: 59    LXI 1
121: 8f    LYI 0xf
122: 0d    L
123: 7f    AI 0xf
124: 6812  JPL 012
126: 5a    LXI 2
127: 8c    LYI 0xc
128: 19    DCM
129: 6812  JPL 012
12b: 08    ICY
12c: 19    DCM
12d: 6812  JPL 012
12f: 08    ICY
130: 19    DCM
131: 6812  JPL 012
133: 8f    LYI 0xf
134: 59    LXI 1
135: 19    DCM
136: 0d    L
137: b0    CI 0x0
138: 6947  JPL 147
13a: 5a    LXI 2
13b: 0d    L
13c: 1c    ROR
13d: 23    RSTC
13e: 0e    ADC
13f: 1c    ROR
140: 1d    ST
141: 7f    AI 0xf
142: 6812  JPL 012
144: 91    LI 0x1
145: 59    LXI 1
146: 1d    ST
147: 5a    LXI 2
148: 84    LYI 0x4
149: 0d    L
14a: 8d    LYI 0xd
14b: 1d    ST
14c: 7f    AI 0xf
14d: d7    JMP 157
14e: 85    LYI 0x5
14f: 0d    L
150: 8e    LYI 0xe
151: 1d    ST
152: 8c    LYI 0xc
153: 90    LI 0x0
154: 1d    ST
155: 6812  JPL 012
157: 85    LYI 0x5
158: 0d    L
159: 8e    LYI 0xe
15a: 1d    ST
15b: b0    CI 0x0
15c: e3    JMP 163
15d: 8f    LYI 0xf
15e: 1d    ST
15f: 59    LXI 1
160: 1d    ST
161: 6812  JPL 012
163: 90    LI 0x0
164: 8c    LYI 0xc
165: 1d    ST
166: 6812  JPL 012
168: 1b    XX
169: 53    XD 3
16a: 54    XYD 4
16b: 80    LYI 0x0
16c: 13    IN
16d: 5a    LXI 2
16e: 8f    LYI 0xf
16f: 1d    ST
170: 54    XYD 4
171: 53    XD 3
172: 1b    XX
173: 52    XD 2
174: 3e04  EN 0x4
176: 3c    RTI
177: 00    NOP
178: 00    NOP
179: 00    NOP
17a: 00    NOP
17b: 00    NOP
17c: 00    NOP
17d: 00    NOP
17e: 00    NOP
17f: 00    NOP
180: fc    JMP 1bc
181: 00    NOP
182: 00    NOP
183: 00    NOP
184: 69c0  JPL 1c0
186: 00    NOP
187: 00    NOP
188: 69dc  JPL 1dc
18a: 00    NOP
18b: 00    NOP
18c: 6a55  JPL 255
18e: 00    NOP
18f: 00    NOP
190: 6a5d  JPL 25d
192: 00    NOP
193: 00    NOP
194: 69f9  JPL 1f9
196: 00    NOP
197: 00    NOP
198: 6a40  JPL 240
19a: 00    NOP
19b: 00    NOP
19c: 6968  JPL 168
19e: 00    NOP
19f: 00    NOP
1a0: fc    JMP 1bc
1a1: 00    NOP
1a2: 00    NOP
1a3: 00    NOP
1a4: fc    JMP 1bc
1a5: 00    NOP
1a6: 00    NOP
1a7: 00    NOP
1a8: fc    JMP 1bc
1a9: 00    NOP
1aa: 00    NOP
1ab: 00    NOP
1ac: fc    JMP 1bc
1ad: 00    NOP
1ae: 00    NOP
1af: 00    NOP
1b0: fc    JMP 1bc
1b1: 00    NOP
1b2: 00    NOP
1b3: 00    NOP
1b4: fc    JMP 1bc
1b5: 00    NOP
1b6: 00    NOP
1b7: 00    NOP
1b8: fc    JMP 1bc
1b9: 00    NOP
1ba: 00    NOP
1bb: 00    NOP
1bc: 52    XD 2
1bd: 3e04  EN 0x4
1bf: 3c    RTI
1c0: 1b    XX
1c1: 53    XD 3
1c2: 54    XYD 4
1c3: 5b    LXI 3
1c4: 80    LYI 0x0
1c5: 0d    L
1c6: 59    LXI 1
1c7: 85    LYI 0x5
1c8: 1d    ST
1c9: 88    LYI 0x8
1ca: 93    LI 0x3
1cb: 1d    ST
1cc: 5b    LXI 3
1cd: 87    LYI 0x7
1ce: 0d    L
1cf: 59    LXI 1
1d0: 1a    STDC
1d1: 5b    LXI 3
1d2: 0d    L
1d3: 59    LXI 1
1d4: 1d    ST
1d5: 54    XYD 4
1d6: 53    XD 3
1d7: 1b    XX
1d8: 52    XD 2
1d9: 3e04  EN 0x4
1db: 3c    RTI
1dc: 1b    XX
1dd: 53    XD 3
1de: 54    XYD 4
1df: 5b    LXI 3
1e0: 88    LYI 0x8
1e1: 0d    L
1e2: 59    LXI 1
1e3: 8a    LYI 0xa
1e4: 1d    ST
1e5: 8d    LYI 0xd
1e6: 93    LI 0x3
1e7: 0a    STIC
1e8: 5b    LXI 3
1e9: 0d    L
1ea: 51    XD 1
1eb: 08    ICY
1ec: 0d    L
1ed: 59    LXI 1
1ee: 8c    LYI 0xc
1ef: 1a    STDC
1f0: 51    XD 1
1f1: 1d    ST
1f2: 53    XD 3
1f3: 1b    XX
1f4: 52    XD 2
1f5: 54    XYD 4
1f6: 3e04  EN 0x4
1f8: 3c    RTI
1f9: 1b    XX
1fa: 53    XD 3
1fb: 54    XYD 4
1fc: 5a    LXI 2
1fd: 81    LYI 0x1
1fe: 0d    L
1ff: 8c    LYI 0xc
200: 1d    ST
201: 82    LYI 0x2
202: 0d    L
203: 8d    LYI 0xd
204: 1d    ST
205: 83    LYI 0x3
206: 0d    L
207: 8e    LYI 0xe
208: 1d    ST
209: 80    LYI 0x0
20a: 0d    L
20b: 8f    LYI 0xf
20c: 1d    ST
20d: 92    LI 0x2
20e: 59    LXI 1
20f: 1d    ST
210: 53    XD 3
211: 1b    XX
212: 52    XD 2
213: 54    XYD 4
214: 3e04  EN 0x4
216: 3c    RTI
217: 00    NOP
218: 00    NOP
219: 00    NOP
21a: 00    NOP
21b: 00    NOP
21c: 00    NOP
21d: 00    NOP
21e: 00    NOP
21f: 00    NOP
220: 00    NOP
221: 00    NOP
222: 00    NOP
223: 00    NOP
224: 00    NOP
225: 00    NOP
226: 00    NOP
227: 00    NOP
228: 00    NOP
229: 00    NOP
22a: 00    NOP
22b: 00    NOP
22c: 00    NOP
22d: 00    NOP
22e: 00    NOP
22f: 00    NOP
230: 00    NOP
231: 00    NOP
232: 00    NOP
233: 00    NOP
234: 00    NOP
235: 00    NOP
236: 00    NOP
237: 00    NOP
238: 00    NOP
239: 00    NOP
23a: 00    NOP
23b: 00    NOP
23c: 00    NOP
23d: 00    NOP
23e: 00    NOP
23f: 00    NOP
240: 1b    XX
241: 53    XD 3
242: 54    XYD 4
243: 5a    LXI 2
244: 89    LYI 0x9
245: 55    XYD 5
246: 3f04  DIS 0x4
248: 25    TSTI
249: cb    JMP 24b
24a: c8    JMP 248
24b: 25    TSTI
24c: cb    JMP 24b
24d: 80    LYI 0x0
24e: 13    IN
24f: 55    XYD 5
250: 1a    STDC
251: 12    INK
252: 1a    STDC
253: 55    XYD 5
254: e5    JMP 265
255: 1b    XX
256: 53    XD 3
257: 54    XYD 4
258: 87    LYI 0x7
259: 55    XYD 5
25a: 3f04  DIS 0x4
25c: e5    JMP 265
25d: 1b    XX
25e: 53    XD 3
25f: 5b    LXI 3
260: 54    XYD 4
261: 8f    LYI 0xf
262: 55    XYD 5
263: 3f04  DIS 0x4
265: 25    TSTI
266: e8    JMP 268
267: e5    JMP 265
268: 25    TSTI
269: e8    JMP 268
26a: 80    LYI 0x0
26b: 13    IN
26c: 55    XYD 5
26d: 1a    STDC
26e: 12    INK
26f: 1a    STDC
270: 14    TYA
271: 55    XYD 5
272: 23    RSTC
273: 0c    ROL
274: be    CI 0xe
275: e5    JMP 265
276: 53    XD 3
277: 1b    XX
278: 52    XD 2
279: 54    XYD 4
27a: 3e04  EN 0x4
27c: 3c    RTI
27d: 00    NOP
27e: 00    NOP
27f: 00    NOP
280: 00    NOP
281: 00    NOP
282: 00    NOP
283: 00    NOP
284: 00    NOP
285: 00    NOP
286: 00    NOP
287: 00    NOP
288: 00    NOP
289: 00    NOP
28a: 00    NOP
28b: 00    NOP
28c: 00    NOP
28d: 00    NOP
28e: 00    NOP
28f: 00    NOP
290: 00    NOP
291: 00    NOP
292: 00    NOP
293: 00    NOP
294: 00    NOP
295: 00    NOP
296: 00    NOP
297: 00    NOP
298: 00    NOP
299: 00    NOP
29a: 00    NOP
29b: 00    NOP
29c: 00    NOP
29d: 00    NOP
29e: 00    NOP
29f: 00    NOP
2a0: 00    NOP
2a1: 00    NOP
2a2: 00    NOP
2a3: 00    NOP
2a4: 00    NOP
2a5: 00    NOP
2a6: 00    NOP
2a7: 00    NOP
2a8: 00    NOP
2a9: 00    NOP
2aa: 00    NOP
2ab: 00    NOP
2ac: 00    NOP
2ad: 00    NOP
2ae: 00    NOP
2af: 00    NOP
2b0: 00    NOP
2b1: 00    NOP
2b2: 00    NOP
2b3: 00    NOP
2b4: 00    NOP
2b5: 00    NOP
2b6: 00    NOP
2b7: 00    NOP
2b8: 00    NOP
2b9: 00    NOP
2ba: 00    NOP
2bb: 00    NOP
2bc: 00    NOP
2bd: 00    NOP
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
