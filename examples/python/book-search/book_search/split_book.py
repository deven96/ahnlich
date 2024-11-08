from pathlib import Path

import ebooklib
from ahnlich_client_py.internals import ai_query
from bs4 import BeautifulSoup
from ebooklib import epub

BASE_DIR = Path(__file__).resolve().parent.parent

def load_epub(file_path):
  
  book = epub.read_epub(file_path)
  return book

def split_into_chapters(book):
  paragraphs = []
  for chapter_num, item in enumerate(book.get_items_of_type(ebooklib.ITEM_DOCUMENT), start=1):
    paragraph_num = 1
    content = item.get_body_content()
    soup = BeautifulSoup(content, 'html.parser')
    
    for p in soup.find_all('p'):
      text = p.text.strip().replace('\n', '')
      if text:
        paragraphs.append((
          ai_query.StoreInput__RawString(text),
          {
            "chapter": ai_query.MetadataValue__RawString(f'{chapter_num}'),
            "paragraph": ai_query.MetadataValue__RawString(f'{paragraph_num}')}
        ))
        paragraph_num += 1
  return paragraphs

def process_epub(file_path):
  book = load_epub(file_path)
  chapters = split_into_chapters(book)
  return chapters

def get_book():
  result = process_epub(f"{BASE_DIR}/book_search/animal_farm.epub")
  return result