package cn.jy.lazydict;

/**
 * 汉字释义
 */
public class Word {
    private final String word;
    private final String strokes;
    private final String pinyin;
    private final String radicals;
    private final String explanation;

    @Override
    public String toString() {
        return pinyin+"<br/><b>"+word+"</b><br/><br/>笔画数："+strokes+"<br/>部首："+radicals+"<br/><br/>"+explanation;
    }

    public Word(String word, String strokes, String pinyin, String radicals, String explanation) {
        this.word = word;
        this.strokes = strokes;
        this.pinyin = pinyin;
        this.radicals = radicals;
        this.explanation = explanation;
    }

    public String getWord() {
        return word;
    }

    public String getStrokes() {
        return strokes;
    }

    public String getPinyin() {
        return pinyin;
    }

    public String getRadicals() {
        return radicals;
    }

    public String getExplanation() {
        return explanation;
    }
}