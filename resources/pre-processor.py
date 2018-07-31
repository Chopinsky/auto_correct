import urllib.request
from pathlib import Path

URL = "http://www.kilgarriff.co.uk/BNClists/lemma.num"
ALPHABET = "abcdefghijklmnopqrstuvwxyz"

WORD_LIMIT_HIGH = 144000
WORD_LIMIT_LOW = 72000
BASE_SCORE = 1270

def init_lemma(override=True, path="lemma.txt"):
    lemma = Path(path)
    if not lemma.is_file() or override:
        urllib.request.urlretrieve(URL, path)


def process_words(override=True, skip_score=False, path="lemma.txt", dest_path="words.txt"):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    with open(path, 'r') as f:
        with open(dest_path, 'a') as df:
            lines = f.readlines()

            for line in lines:
                word_info = line.split(' ')

                if word_info[2] == "": continue

                if skip_score:
                    word = word_info[2]
                else:
                    word = word_info[2] + "," + word_info[1]

                df.write(word + '\n')

            df.close()

        f.close()


def process_n_cut(override=True, cut_level=2, path="freq_50k.txt", dest_path="freq_50k.txt"):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    with open(path, 'r') as f:
        with open(dest_path, 'a') as df:
            count = 0
            lines = f.readlines()

            for line in lines:
                if line == "" or line == "word,count\n":
                    continue

                word_info = line.replace("\n", "").split(',')

                if word_info[0] != "" and word_info[1] != "":
                    score = int(int(word_info[1]) / BASE_SCORE)
                    df.write(word_info[0] + "," + str(score) + "\n")

                count += 1

                if cut_level == 0  and count == WORD_LIMIT_LOW:
                    break
                elif cut_level == 1 and count == WORD_LIMIT_HIGH:
                    break

            print(count)
            df.close()

        f.close()


def process_n_save(override=True, path="freq_50k.txt", dest_path="freq_50k_precalc.txt"):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    dict = {}
    with open(path, 'r') as f:
        with open(dest_path, 'a') as df:
            print("Info: process begins...")

            count = 0
            lines = f.readlines()

            for line in lines:
                if line == "" or line == "word,count\n":
                    continue

                count += 1
                word_info = line.replace("\n", "").split(',')

                if word_info[0] != "" and word_info[1] != "" and word_info[0] not in dict:
                    dict[word_info[0]] = word_info[1]

            print(len(dict))
            for (word, score) in dict.items():
                neighbors = words_in_one_edit(word, dict)
                result = word + "^"

                for neighbor in neighbors:
                    if len(neighbor) > 0:
                        result = result + neighbor + ";"

                result = result + "\n"
                df.write(result)

            df.close()

        f.close()

    return dict


def words_in_one_edit(source, dict):
    result = set()
    length = len(source)

    for i in range(length+1):
        if length > 1:
            # delete...
            delete = source[0:i] + source[i+1:]
            check_and_add(delete, source, dict, result)

            # swap...
            if i+2 <= length:
                swap = source[0:i] + source[i+1] + source[i] + source[i+2:]
                check_and_add(swap, source, dict, result)

        for letter in ALPHABET:
            # insert...
            insert = source[0:i] + letter + source[i:]
            check_and_add(insert, source, dict, result)

            # replace...
            replace = source[0:i] + letter + source[i+1:]
            check_and_add(replace, source, dict, result)

    return result


def check_and_add(target, source, dict, result):
    if target != source and target in dict and target not in result:
        result.add(target)


if __name__ == '__main__':
    #init_lemma(False)
    #process_words(dest_path="words2.txt")
    #process_n_cut(cut_level=0, dest_path="uniq_low.txt")

    dict = process_n_save()

    #result = words_in_one_edit("word", dict)
    #print(result)

    print("\nDone...")