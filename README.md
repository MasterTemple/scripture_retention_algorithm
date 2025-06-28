This method is taken from Tom Frost

If the current date is within 7 weeks from the start date, then it is a daily verse.
If the current date is past 7 weeks from the start date AND it is within 35 (28 + 7) weeks, then it is a weekly verse.
If the current date is past 35 weeks from the start date AND it is within 336 (35 + (84 * 4)) weeks, then it is a monthly verse.

---

I need to figure out all verses that I would need to do in a month, so I can organize.
I can do this by looking at all verses required at the start of the month, and at the end of the month and then creating a union.

This is for the monthly ones (which means I don't need to worry about doing a very multiple times in a month).

I need to determine:

### Daily Verses

By each day

### Weekly Verses

Which is all verses in the weekly category
Divide it by every week
Because new verses are only added at the beginning/end of a week

### Monthly Verses

Which is all current verses in the monthly category plus all 

Okay, so at the beginning of the month I need to figure out all verses that will be in the monthly verse category
This means all verses that are currently monthly verses and all weekly verses that are completed in 4 weeks from the start of the month.

---

Basically I just need 1 function that calculates all verses for the month, as well as the day that I say them

and then I either just
1. Add new daily verses
2. Always use the first Sunday of the month (basically, if I add more verses, just recalculate from the first Sunday of the "month"; note: actual months are meaningless, they are just 4-week periods of time with intervals corresponding to the start date)

so the problem with monthly planning is that I only do something once every 4 weeks
so perhaps what i can do is some modulus math to determine every nth week (where n has 4 weeks in between, based on start date, and then treat it like it is weekly)

for daily, it is everything
for weekly, i just say "what is to be done weekly" and then divide it into 7 groups
