use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub enum ViewScope {
    Day(NaiveDate),
    Week(NaiveDate),  // Start of week
    Month(NaiveDate), // Start of month
}

#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub scope: ViewScope,
}

impl DateRange {
    pub fn day(date: NaiveDate) -> Self {
        Self {
            start: date,
            end: date,
            scope: ViewScope::Day(date),
        }
    }

    pub fn week(start_of_week: NaiveDate) -> Self {
        let end = start_of_week + chrono::Duration::days(6);
        Self {
            start: start_of_week,
            end,
            scope: ViewScope::Week(start_of_week),
        }
    }

    pub fn month(year: i32, month: u32) -> Self {
        let start = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid year/month");
        let end = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .expect("Invalid date calculation")
        .pred_opt()
        .expect("Invalid month end calculation");

        Self {
            start,
            end,
            scope: ViewScope::Month(start),
        }
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.start && date <= self.end
    }

    pub fn days(&self) -> impl Iterator<Item = NaiveDate> {
        let start = self.start;
        let end = self.end;
        (0..=(end - start).num_days()).map(move |i| start + chrono::Duration::days(i))
    }
}
