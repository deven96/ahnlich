from pathlib import Path

import ebooklib
from bs4 import BeautifulSoup
from ebooklib import epub

from ahnlich_client_py.grpc import keyval, metadata
from ahnlich_client_py.grpc.ai import query as ai_query

BASE_DIR = Path(__file__).resolve().parent.parent
EPUB_FILE = BASE_DIR / "book_search" / "animal_farm.epub"


def load_epub(file_path: Path) -> epub.EpubBook:
    return epub.read_epub(str(file_path))


def split_into_chapters(book):
    paragraphs = []
    for chapter_num, item in enumerate(
        book.get_items_of_type(ebooklib.ITEM_DOCUMENT), start=1
    ):
        paragraph_num = 1
        content = item.get_body_content()
        soup = BeautifulSoup(content, "html.parser")

        for p in soup.find_all("p"):
            text = p.text.strip().replace("\n", "")
            if text:
                paragraphs.append(
                    ai_query.StoreEntry(
                        key=keyval.StoreInput(raw_string=text),
                        value={
                                "chapter": metadata.MetadataValue(
                                    raw_string=f"{chapter_num}"
                                ),
                                "paragraph": metadata.MetadataValue(
                                    raw_string=f"{paragraph_num}"
                                ),
                        },
                    )
                )
                paragraph_num += 1
    return paragraphs


def get_book() -> list[keyval.StoreEntry]:
    book = load_epub(EPUB_FILE)
    return split_into_chapters(book)
