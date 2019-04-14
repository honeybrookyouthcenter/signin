using System;
using System.Text.RegularExpressions;

namespace YouthCenterSignIn.Logic.Data
{
    public class Guardian : NotifyBase
    {
        public Guardian()
        { }

        public Guardian(string name, string phoneNumber, string email = null, DateTime? lastUpdated = null)
        {
            Name = name;
            PhoneNumber = phoneNumber;
            Email = email;
            LastUpdated = lastUpdated;
        }

        string name;
        public string Name
        {
            get => name;
            set { name = value; OnPropertyChanged(); }
        }

        string phoneNumber;
        public string PhoneNumber
        {
            get => phoneNumber;
            set { phoneNumber = value; OnPropertyChanged(); }
        }

        string email;
        public string Email
        {
            get => email;
            set { email = value; OnPropertyChanged(); }
        }

        public DateTime? LastUpdated { get; set; }

        public bool IsInfoExpired => LastUpdated > DateTime.Today.AddMonths(-3);

        public bool IsValid(out string issues)
        {
            issues = null;
            if (string.IsNullOrWhiteSpace(Name))
                issues += "name";
            if (string.IsNullOrWhiteSpace(PhoneNumber))
                issues += issues == null ? "phone number" : " and phone number";

            if (issues != null)
                issues = $"Please enter your guardian's {issues}.";

            return issues == null;
        }

        const string NameLabel = "Guardian's Name: ";
        const string PhoneLabel = "Guardian's Phone: ";
        const string EmailLabel = "Guardian Email: ";
        const string LastUpdatedLabel = "Last updated: ";

        readonly static Regex NameRegex = new Regex(GetLabelExpression(NameLabel), RegexOptions.Compiled);
        readonly static Regex PhoneRegex = new Regex(GetLabelExpression(PhoneLabel), RegexOptions.Compiled);
        readonly static Regex EmailRegex = new Regex(GetLabelExpression(EmailLabel), RegexOptions.Compiled);
        readonly static Regex LastUpdatedRegex = new Regex(GetLabelExpression(LastUpdatedLabel), RegexOptions.Compiled);

        static string GetLabelExpression(string label) => $@"(?<={label}).*";

        public override string ToString()
        {
            LastUpdated = DateTime.Today;

            return $"{NameLabel}{Name}\r\n" +
                   $"{PhoneLabel}{PhoneNumber}\r\n" +
                   $"{EmailLabel}{Email}\r\n" +
                   $"{LastUpdatedLabel}{LastUpdated?.ToLongDateString()}";
        }

        public static Guardian FromNotes(string notes)
        {
            if (notes == null)
                notes = "";

            string GetMatch(Regex regex) => regex.Match(notes).Value.Trim();

            var name = GetMatch(NameRegex);
            var phone = GetMatch(PhoneRegex);
            var email = GetMatch(EmailRegex);

            var lastUpdatedString = GetMatch(LastUpdatedRegex);
            var lastUpdated = ParseDateTime(lastUpdatedString);

            return new Guardian(name, phone, email, lastUpdated);
        }

        static DateTime? ParseDateTime(string lastUpdatedString)
        {
            try
            {
                if (string.IsNullOrWhiteSpace(lastUpdatedString))
                    return null;

                return DateTime.Parse(lastUpdatedString);
            }
            catch
            {
                return null;
            }
        }
    }
}
