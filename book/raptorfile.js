/*
  Language: Raptorfile
  Requires: bash.js
  Author: Alexis Hénaut <alexis@henaut.net>
  Author: Christian Iversen <ci@iversenit.dk>
  Description: language definition for Raptor files
  Website: https://github.com/chrivers/raptor
  Category: config
*/

/** @type LanguageFn */
export default function(hljs) {
    const KEYWORDS = [
        "FROM",
        "ENV",
        "MOUNT",
    ];
    return {
        name: 'raptorfile',
        aliases: [ 'raptor' ],
        case_insensitive: false,
        keywords: KEYWORDS,
        contains: [
            hljs.HASH_COMMENT_MODE,
            hljs.APOS_STRING_MODE,
            hljs.QUOTE_STRING_MODE,
            hljs.NUMBER_MODE,
            {
                beginKeywords: 'RENDER WRITE MKDIR COPY INCLUDE RUN WORKDIR ENTRYPOINT CMD',
                starts: {
                    end: /[^\\]$/,
                    subLanguage: 'bash'
                }
            }
        ],
        illegal: '</'
    };
}
