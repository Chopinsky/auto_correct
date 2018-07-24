import urllib.request
from pathlib import Path

URL = "http://www.kilgarriff.co.uk/BNClists/lemma.num"

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


def process_n_save(override=True, path="freq_50k.txt", dest_path="freq_50k_proc.txt"):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    with open(path, 'r') as f:
        with open(dest_path, 'a') as df:
            print("Info: process begins...")

            count = 0
            dict = {}
            lines = f.readlines()

            for line in lines:
                if line == "" or line == "word,count\n":
                    continue

                count += 1
                word_info = line.replace("\n", "").split(',')

                if word_info[0] != "" and word_info[1] != "" and word_info[0] not in dict:
                    dict[word_info[0]] = word_info[1]

            print(count)
            print(dict)

            df.close()

        f.close()


if __name__ == '__main__':
    #init_lemma(False)
    #process_words(dest_path="words2.txt")
    #process_n_cut(cut_level=0, dest_path="uniq_low.txt")

    process_n_save()
    print("\nDone!")