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
            .waitForElementPresent('#POPUP_CONTAINER')
            .pause(5000)
        //var f = browser.element('css selector', '._25r3t_lrPF3M6zD2YkWvZU');
            .frame(1)
            .waitForElementVisible('button.AnimatedForm__submitButton.m-full-width')
            .click('partial link text', 'User Agreement')
            .setValue('input[name=username]', userName)
            .setValue('input[name=password]', oldPasswd)
            .pause(5000)
            .click('button.AnimatedForm__submitButton.m-full-width')
        //var frame = browser.element('iframe._25r3t_lrPF3M6zD2YkWvZU');

            // .getAttribute('iframe._25r3t_lrPF3M6zD2YkWvZU', 'id', (result) => {
            //     console.log(result.value);
            //     browser
            //         .frame(result.value)
            //         .waitForElementPresent('button[type=submit]')
            //         .setValue('input[name=username]', userName)
            //         .setValue('input[name=password]', oldPasswd)
            //         .click('button[type=submit]')
            // })
            .pause(5000)
            .frame(null)
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