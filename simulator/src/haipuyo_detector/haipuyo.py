# https://puyo-camp.jp/posts/86154

import os


color = 'rgbyp'
puyoai_color = 'rbyg'


def lacking_color(tumo):
    for i in color:
        if i not in tumo:
            return i


def normalize(tumo, lack):
    idx = color.index(lack)
    source = color[:idx] + color[idx + 1:]
    target = puyoai_color
    return tumo.translate(str.maketrans(source, target))


def tumo_hash(tumo):
    hsh = 0
    for t in tumo:
        hsh <<= 2
        hsh += puyoai_color.index(t)
    return hsh


def full_tumo_hash(tumo):
    assert len(tumo) == 256
    return [tumo_hash(tumo[32 * i: 32 * (i + 1)][::-1]) for i in range(8)]


if __name__ == '__main__':
    base = os.path.dirname(os.path.abspath(__file__))
    file_in = os.path.normpath(os.path.join(base, 'haipuyo.txt'))
    file_out = os.path.normpath(os.path.join(base, 'haipuyo.rs'))

    with open(file_in, mode='r') as f:
        cnt = [0 for _ in range(len(color))]
        keys = []
        tumos = []
        for tumo in f.readlines():
            tumo = tumo.rstrip()
            lack = lacking_color(tumo)
            norm = normalize(tumo, lack)
            hsh_head = tumo_hash(norm[:16])
            hsh_full = full_tumo_hash(norm[12:] + norm[:12])
            keys.append(hsh_head)
            tumos.append((hsh_head, hsh_full))

    keys.sort()
    tumos.sort()

    tumos_trans = [[] for _ in range(8)]
    for i in tumos:
        i = i[1]
        for j in range(8):
            tumos_trans[j].append(i[j])

    with open(file_out, mode='w') as f:
        # keys
        f.write("pub const HAIPUYO_KEYS: &'static [u32] = &[")
        f.write(', '.join(map(lambda x: f'{x}u32', keys)))
        f.write('];\n')

        # tumos
        for i in range(8):
            f.write(f"\npub const HAIPUYO_{i}: &'static [u64] = &[")
            f.write(', '.join(map(lambda x: f'{x}u64', tumos_trans[i])))
            f.write('];\n')
