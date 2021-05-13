module.exports = {
    'reddit.com' : function(browser) {
        const url = process.argv[6];
        const userName = process.argv[7];
        const oldPasswd = process.argv[8];
        const newPasswd = process.argv[9];
    
        browser
            .url(url)
            .waitForElementPresent('body')
            .waitForElementPresent('partial link text', 'Log In')
            .click('partial link text', 'Log In')
            .waitForElementPresent('button[type=submit]')
            .setValue('#loginUsername', userName)
            .setValue('#loginPassword', oldPasswd)
            .click('button[type=submit]')
            .waitForElementPresent('div.header-user-dropdown')
            .click('div.header-user-dropdown')
            .waitForElementPresent('partial link text', 'User Settings')
            .click('partial link text', 'User Settings')
            .waitForElementPresent('button._2iuoyPiKHN3kfOoeIQalDT._2tU8R9NTqhvBrhoNAXWWcP.HNozj_dKjQZ59ZsfEegz8')
            .click('button._2iuoyPiKHN3kfOoeIQalDT._2tU8R9NTqhvBrhoNAXWWcP.HNozj_dKjQZ59ZsfEegz8')
            .waitForElementPresent('input[name=old_password]')
            .setValue('input[name=old_password]', oldPasswd)
            .setValue('input[name=password]', newPasswd)
            .setValue('input[name=password2]', newPasswd)
            .click('button[type=submit]')
            .assert.containsText('User settings')
            .end();
    }
};