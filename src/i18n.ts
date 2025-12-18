import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zh from './locales/zh'
import zhTW from './locales/zh-TW'
import ru from './locales/ru'
import ja from './locales/ja'
import ko from './locales/ko'
import es from './locales/es'

// Detect default locale based on browser settings
function getDefaultLocale(): string {
    const saved = localStorage.getItem('locale')
    if (saved) return saved

    const browserLang = navigator.language.toLowerCase()
    if (browserLang.startsWith('zh-tw') || browserLang.startsWith('zh-hant')) return 'zh-TW'
    if (browserLang.startsWith('zh')) return 'zh'
    if (browserLang.startsWith('ja')) return 'ja'
    if (browserLang.startsWith('ko')) return 'ko'
    if (browserLang.startsWith('ru')) return 'ru'
    if (browserLang.startsWith('es')) return 'es'
    return 'en'
}

const i18n = createI18n({
    legacy: false, // Use Composition API mode
    locale: getDefaultLocale(),
    fallbackLocale: 'en',
    messages: {
        en,
        zh,
        'zh-TW': zhTW,
        ru,
        ja,
        ko,
        es
    }
})

export default i18n

