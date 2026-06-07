# History of Locks

## Chest above Cavalorn's cottage (near two New Camp thieves)

**Rules:**
```
1: 3r, 5l
2: 3l, 4l
3: 4r, 6r
4: 1l, 3l
5: -
6: -
```

**Start:** `[1, 4, 5, 2, 1, 4]`

**Solution:** (35 clicks — verified optimal by BFS via the `locks` solver; the
original hand-entry started `5: 6x D`, which drove plate 5 from 1 below the wall)
```
3: 1x D
5: 1x A
1: 1x A
4: 1x A
3: 1x D
5: 1x A
1: 1x A
4: 1x A
3: 1x D
5: 1x A
1: 1x A
4: 1x A
5: 1x A
1: 1x A
4: 1x A
5: 1x A
1: 1x A
4: 1x A
5: 1x A
1: 1x A
4: 1x A
5: 1x A
1: 1x A
5: 1x A
1: 1x A
5: 1x A
1: 1x A
5: 3x A
6: 1x A
3: 1x D
6: 3x A
```

## Cave near Cavalorn's cottage

**Rules:**
```
1: 2l, 3r
2: -
3: 1r, 5l
4: 6r, 2l, 3l
5: 4l, 6r
6: 1r
```

**Start:** `[2, 4, 5, 7, 5, 5]`

**Solution:** (41 clicks — verified optimal by BFS via the `locks` solver)
```
1: 1x D
2: 4x D
4: 1x D
2: 1x D
4: 1x D
2: 1x D
4: 1x D
2: 1x D
5: 1x D
6: 1x A
1: 1x D
2: 1x D
4: 1x D
2: 1x D
6: 1x A
1: 1x D
2: 1x D
4: 1x D
2: 1x D
6: 1x A
1: 1x D
2: 1x D
4: 1x D
6: 1x A
1: 1x D
4: 1x D
6: 1x A
3: 1x D
5: 1x D
6: 1x A
3: 1x D
5: 1x D
6: 1x A
3: 1x D
5: 1x D
6: 3x A
```

## Door to tower near Cavalorn's cottage

**Rules:**
```
1: -
2: 3l, 4r, 5r
3: 4r
4: -
5: 6l, 3l, 2r, 1l
6: 2r, 1l
```

**Start:** `[5, 6, 2, 2, 1, 1]`

**Solution:** (57 clicks — verified optimal by BFS via the `locks` solver)
```
1: 4x D
2: 1x A
3: 4x A
4: 1x D
3: 1x A
4: 6x D
5: 1x D
1: 1x D
2: 1x A
4: 1x D
5: 1x D
1: 1x D
2: 1x A
4: 1x D
5: 1x D
1: 1x D
2: 1x A
4: 1x D
5: 1x D
1: 1x D
2: 1x A
4: 1x D
5: 1x D
1: 1x D
2: 1x A
4: 1x D
5: 1x D
1: 1x D
2: 1x A
4: 1x D
6: 1x D
1: 1x D
2: 1x A
4: 1x D
5: 1x D
1: 1x D
2: 1x A
6: 1x D
1: 1x D
2: 1x A
5: 1x D
1: 1x D
2: 1x A
6: 3x D
```

## First chest in the tower

**Rules:**
```
1: 2l
2: 5r
3: 4l, 5l, 6r
4: -
5: -
6: 2l
```

**Start:** `[3, 1, 7, 4, 3, 2]`

**Solution:**
```
3: 1x D
2: 3x A
6: 3x A
5: 6x D
2: 6x A
6: 2x A
5: 5x D
3: 2x D
1: 1x A
4: 3x D
```

## Second chest in the tower

**Rules:** (tumbler 4 corrected after in-game re-measurement: `5r`, not `3r`)
```
1: 3r, 6l
2: -
3: 1r, 4l, 6r
4: 2r, 5r, 6l
5: -
6: 3l
```

**Start:** `[5, 3, 6, 7, 2, 7]`

**Solution:** (52 clicks — verified optimal by BFS via the `locks` solver; the
previous entry was 56 clicks and ended with plate 6 at 5 — pre-dated the tumbler-4
`5r` re-measurement)
```
1: 1x A
2: 4x A
4: 1x D
2: 1x A
3: 1x D
1: 1x A
5: 1x A
4: 1x D
2: 1x A
3: 1x D
1: 1x A
5: 1x A
4: 1x D
2: 1x A
3: 1x D
1: 1x A
5: 1x A
4: 1x D
2: 1x A
3: 1x D
1: 1x A
5: 1x A
4: 1x D
2: 1x A
3: 1x D
1: 1x A
5: 1x A
4: 1x D
2: 1x A
3: 1x D
5: 1x A
4: 1x D
1: 1x A
2: 1x A
5: 1x A
4: 1x D
2: 1x A
3: 1x D
5: 1x A
4: 1x D
3: 1x D
5: 1x A
4: 1x D
5: 1x A
4: 1x D
5: 3x A
6: 1x A
```

## kharim chest

**Rules:**
```
1: -
2: -
3: 2l,6r
4: -
5: 2l
6: 5r,3l
7: 6r,4l,1l
```

**Start:** `[3, 3, 4, 2, 3, 7, 1]`

**Solution:** (24 clicks — solved by locks)
```
1: 4x A
2: 2x A
3: 2x D
4: 5x A
5: 1x A
3: 1x D
5: 3x A
6: 3x D
7: 3x A
```

## chest near arena big home

**Rules:**
```
1: 4l
2: 1l,4r,6l
3: 6l
4: 2r,6r
5: 2r,6l
6: 4r
```

**Start:** `[5, 6, 1, 3, 2, 1]`

**Solution:** (30 clicks — solved by locks)
```
1: 4x D
2: 1x D
1: 1x D
2: 1x D
1: 1x D
2: 1x D
1: 1x D
2: 1x D
1: 1x D
2: 1x D
1: 1x D
3: 3x A
5: 1x A
2: 1x D
4: 1x A
2: 1x D
4: 1x A
2: 1x D
5: 1x A
4: 1x A
6: 1x D
4: 1x A
6: 3x D
```

## scatty chest

**Rules:**
```
1: 4r
2: 5r
3: 5l
4: 2r,3l,5l
5: -
```

**Start:** `[6, 3, 2, 1, 7]`

**Solution:** (31 clicks — solved by locks)
```
2: 2x D
3: 4x A
5: 1x A
3: 1x A
5: 1x A
4: 1x A
1: 1x D
5: 1x A
2: 1x D
5: 1x A
3: 1x A
5: 1x A
4: 1x A
1: 1x D
5: 1x A
2: 1x D
5: 1x A
3: 1x A
5: 1x A
4: 1x A
5: 1x A
4: 1x A
5: 1x A
4: 1x A
5: 3x A
```

## second chest behind ubert in mine

**Rules:**
```
1: 3l
2: 1l,3r,5r
3: 1r,2r,4r
4: -
5: 4r,3l
```

**Start:** `[1, 6, 2, 3, 3]`

**Solution:** (62 clicks — solved by locks)
```
2: 1x D
1: 1x D
2: 1x D
1: 1x D
3: 1x A
1: 1x D
3: 1x A
1: 1x D
3: 1x A
4: 5x D
5: 1x A
1: 1x D
2: 1x D
1: 1x D
4: 1x D
5: 1x A
2: 1x D
1: 1x D
3: 1x A
4: 2x D
5: 1x A
1: 1x D
2: 1x D
1: 1x D
4: 1x D
5: 1x A
2: 1x D
1: 1x D
3: 1x A
4: 2x D
5: 1x A
1: 1x D
2: 1x D
1: 1x D
4: 1x D
5: 1x A
2: 1x D
1: 1x D
3: 1x A
4: 2x D
5: 1x A
1: 1x D
2: 1x D
1: 1x D
4: 1x D
5: 1x A
2: 1x D
3: 2x A
4: 3x D
5: 3x A
```
