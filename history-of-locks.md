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

## first chest behind ubert mine

**Rules:**
```
1: 2r,5l
2: 1l,4l,6r
3: -
4: 2l,5l,6r
5: 3r
6: 5r
```

**Start:** `[7, 6, 2, 6, 4, 2]`

**Solution:** (42 clicks — solved by locks)
```
1: 3x D
2: 1x D
3: 5x A
5: 1x D
1: 1x D
3: 1x A
5: 1x D
3: 1x A
5: 1x D
3: 1x A
5: 1x D
3: 1x A
5: 1x D
3: 1x A
5: 1x D
3: 1x A
5: 1x D
3: 1x A
6: 1x A
4: 1x D
5: 1x D
3: 1x A
5: 1x D
3: 1x A
6: 1x A
4: 1x D
5: 1x D
3: 1x A
5: 1x D
6: 1x A
4: 1x D
5: 2x D
6: 3x A
```

## aaron chest

**Rules:**
```
1: 6l
2: -
3: 1r,2l,4r
4: 3l
5: 6r
6: 4r,2r
```

**Start:** `[1, 5, 6, 6, 6, 5]`

**Solution:** (60 clicks — solved by locks)
```
1: 4x A
2: 4x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
4: 1x D
6: 1x A
1: 1x A
2: 1x D
3: 1x D
2: 1x D
6: 1x A
1: 1x A
2: 1x D
3: 1x D
2: 1x D
6: 1x A
1: 1x A
2: 1x D
4: 1x D
6: 1x A
1: 1x A
2: 1x D
3: 1x D
2: 1x D
6: 1x A
1: 1x A
2: 1x D
4: 1x D
6: 1x A
1: 1x A
2: 1x D
3: 1x D
2: 1x D
6: 1x A
1: 1x A
2: 1x D
4: 1x D
6: 1x A
2: 1x D
4: 1x D
5: 1x D
6: 1x A
2: 1x D
4: 1x D
5: 1x D
6: 3x A
```

## first cor kalom chest

**Rules:**
```
1: 4r
2: 1r,4r
3: 2l,1r
4: -
```

**Start:** `[7, 7, 6, 1]`

**Solution:** (15 clicks — solved by locks)
```
4: 1x A
2: 1x D
1: 1x A
2: 1x D
1: 1x A
2: 1x D
1: 1x A
2: 1x D
1: 1x A
2: 1x D
3: 2x D
4: 3x A
```

## second cor kalom chest

**Rules:**
```
1: 3r
2: -
3: 2l,5l
4: 1l,3r
5: 2l,4r
6: -
```

**Start:** `[6, 5, 5, 7, 1, 4]`

**Solution:** (59 clicks — solved by locks)
```
1: 1x A
2: 4x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
3: 1x D
2: 1x D
5: 1x D
2: 1x D
4: 1x A
1: 1x A
3: 1x D
2: 1x D
3: 1x D
2: 1x D
5: 1x D
2: 1x D
4: 1x A
1: 1x A
3: 1x D
2: 1x D
5: 1x D
2: 1x D
3: 1x D
2: 1x D
4: 1x A
1: 1x A
5: 1x D
2: 1x D
3: 1x D
2: 1x D
4: 1x A
1: 1x A
5: 1x D
2: 1x D
3: 1x D
2: 1x D
4: 1x A
5: 1x D
2: 1x D
3: 1x D
2: 1x D
4: 1x A
5: 1x D
2: 1x D
3: 1x D
2: 1x D
4: 1x A
5: 3x D
```

## bridge near new camp

**Rules:**
```
1: 2r,3l,4l,5r,6r
2: 1l,5l
3: -
4: 5l
5: 2r,4l
6: -
```

**Start:** `[6, 5, 2, 5, 7, 4]`

**Solution:** (25 clicks — solved by locks)
```
2: 1x A
1: 1x A
3: 4x A
5: 1x D
1: 1x A
5: 1x D
2: 1x A
4: 2x D
5: 1x D
2: 1x A
4: 2x D
5: 1x D
2: 1x A
4: 2x D
5: 3x D
6: 2x D
```

## Left chest near cain body flooded mine

**Rules:**
```
1: 4l
2: 1r,3l,5l
3: 1l,2r
4: 3l,5l
5: 1r,2l
```

**Start:** `[4, 5, 6, 5, 1]`

**Solution:** (34 clicks — solved by locks)
```
1: 2x D
3: 4x D
4: 1x D
1: 1x D
2: 1x A
3: 1x D
4: 1x D
1: 1x D
2: 1x A
4: 1x D
1: 1x D
2: 1x A
4: 1x D
1: 1x D
2: 1x A
4: 1x D
1: 1x D
3: 1x D
4: 1x D
1: 1x D
3: 1x D
4: 1x D
1: 1x D
3: 1x D
4: 3x D
5: 3x D
```

## Orc cementary near final Baal Lukora

**Rules:**
```
1: -
2: -
3: -
4: 1l,2r,3l
5: 1l
6: 1l,4l
```

**Start:** `[4, 5, 6, 1, 5, 1]`

**Solution:** (29 clicks — solved by locks)
```
1: 3x A
2: 4x D
3: 1x A
4: 1x A
1: 1x A
2: 1x D
3: 1x A
4: 1x A
1: 1x A
2: 1x D
3: 1x A
4: 1x A
1: 1x A
2: 1x D
3: 1x A
4: 1x A
1: 1x A
4: 1x A
1: 1x A
4: 1x A
5: 1x D
6: 3x A
```

## Trial of fire chest

**Rules:**
```
1: 5r,6l
2: 6r
3: 1l
4: 1l
5: 6l
6: 3l
```

**Start:** `[5, 1, 7, 7, 6, 4]`

**Solution:** (41 clicks — solved by locks)
```
1: 3x D
3: 5x D
5: 1x A
1: 1x D
3: 1x D
5: 1x A
1: 1x D
4: 1x D
5: 1x A
1: 1x D
4: 1x D
5: 1x A
1: 1x D
4: 1x D
5: 1x A
1: 1x D
5: 1x A
1: 1x D
5: 1x A
1: 1x D
5: 1x A
1: 1x D
5: 1x A
1: 1x D
5: 1x A
2: 1x A
6: 1x D
2: 1x A
3: 1x D
6: 1x D
2: 1x A
3: 1x D
6: 3x D
```

## Wood Camp near zombie stone circle

**Rules:**
```
1: 6l
2: 1r
3: 7r
4: 1r,7l
5: 4l,7r
6: 1r
7: 6r,1r
```

**Start:** `[7, 5, 1, 7, 6, 5, 6]`

**Solution:** (30 clicks — solved by locks)
```
2: 1x D
1: 1x A
3: 1x A
7: 1x D
1: 1x A
3: 1x A
7: 1x D
3: 1x A
6: 1x A
7: 1x D
4: 1x D
5: 1x D
4: 1x D
5: 1x D
4: 1x D
6: 4x A
7: 1x D
4: 1x D
6: 2x A
7: 1x D
4: 1x D
6: 2x A
7: 3x D
```

## First chest stone fortress

**Rules:**
```
1: 2r
2: -
3: 1r,4l,6l
4: 6l
5: -
6: 5l,1r
```

**Start:** `[5, 7, 7, 6, 3, 1]`

**Solution:** (34 clicks — solved by locks)
```
2: 1x D
1: 1x A
2: 1x D
1: 1x A
2: 6x D
3: 1x D
1: 1x A
2: 1x D
4: 1x D
3: 1x D
1: 1x A
2: 1x D
4: 1x D
3: 1x D
1: 1x A
4: 1x D
5: 2x D
6: 1x D
1: 1x A
4: 1x D
5: 1x D
6: 1x D
1: 1x A
4: 1x D
5: 1x D
6: 3x D
```

## second chect stone fortress

**Rules:**
```
1: 3r,5l
2: 5l
3: -
4: 1l,2r
5: 3r,1l,2l
6: 4l,1l
```

**Start:** `[2, 2, 6, 6, 4, 7]`

**Solution:** (28 clicks — solved by locks)
```
1: 1x D
2: 4x A
3: 2x A
4: 1x D
1: 1x D
2: 1x A
3: 1x A
4: 1x D
1: 1x D
2: 1x A
3: 1x A
4: 1x D
1: 1x D
3: 1x A
4: 1x D
1: 1x D
3: 1x A
4: 1x D
1: 1x D
5: 1x D
1: 1x D
6: 3x D
```

## Chest on top stone fortress

**Rules:**
```
1: 5r
2: 3l,6r
3: 2r,5r,6l
4: 5r
5: 3l,4r,6l
6: -
```

**Start:** `[2, 7, 4, 7, 1, 6]`

**Solution:** (58 clicks — solved by locks)
```
1: 2x A
2: 3x D
4: 2x D
5: 1x A
2: 1x D
4: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
2: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
2: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
3: 1x A
4: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
2: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
3: 1x A
4: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
3: 1x A
4: 1x D
6: 1x A
5: 1x A
4: 1x D
6: 1x A
3: 1x A
4: 1x D
6: 1x A
5: 1x A
6: 1x A
5: 1x A
6: 1x A
5: 1x A
6: 3x A
```

## under water chest

**Rules:**
```
1: 2r,4r
2: 1r,3r
3: 1r
4: -
```

**Start:** `[2, 5, 7, 7]`

**Solution:** (22 clicks — solved by locks)
```
2: 1x D
3: 1x A
2: 1x D
3: 1x A
2: 1x D
3: 1x A
2: 1x D
4: 1x D
1: 1x A
2: 1x D
4: 1x D
1: 1x A
2: 1x D
4: 1x D
1: 1x A
4: 1x D
1: 1x A
4: 1x D
1: 1x A
4: 3x D
```

## Lockpicking II: Chest inside new camp near buster

**Rules:**
```
1: 5l
2: 5r
3: -
4: 1l,3r,6r
5: 1l,2r,3l
6: 5l
```

**Start:** `[5, 4, 3, 1, 2, 2]`

**Solution:** (16 clicks — solved by locks)
```
1: 1x D
2: 3x A
3: 2x D
4: 1x A
3: 1x D
4: 1x A
3: 1x D
4: 1x A
3: 1x D
5: 3x D
6: 1x D
```

## Aarlan prison cage

**Rules:**
```
1: 6l
2: 3r,5l
3: 4l
4: 5l
5: 4l
6: 4r
7: 1l,3l
```

**Start:** `[1, 1, 3, 1, 7, 6, 7]`

**Solution:** (13 clicks — solved by locks)
```
2: 3x A
3: 5x D
6: 2x D
7: 3x D
```
