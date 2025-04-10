use note::parse_note;
use std::fs;

fn main() {
    let html = "<div class='law-block'><div class='law-block-content-multiple'><p class='law-block-chapter-num'><span class='law-block-chapter'>民法</span>第<span class='law-block-num'>877-1</span>條</p><ul class='law-block-lines'><li class='law-block-line'>以建築物設定抵押權者，於法院拍賣抵押物時，其抵押物存在<span style='color:hsl(30,75%,60%);'>所必要之權利得讓與者</span>，應併付拍賣。但抵押權人對於該權利賣得之價金，無優先受清償之權。</li><li class='law-indent'>我愛你</li></ul></div></div>";
    let x = parse_note(html);
    std::fs::write("law.json", serde_json::to_string(&x).unwrap());
}
