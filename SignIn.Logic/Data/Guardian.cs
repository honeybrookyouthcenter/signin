using System;
using System.Linq;
using System.Text.RegularExpressions;
using System.Xml;

namespace SignIn.Logic.Data
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

        public bool IsInfoExpired => LastUpdated == null || LastUpdated < DateTime.Today.AddMonths(-6);

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

        internal void Clear()
        {
            Name = "";
            PhoneNumber = "";
            Email = "";
        }

        const string NameLabel = "Guardian's Name: ";
        const string PhoneLabel = "Guardian's Phone: ";
        const string EmailLabel = "Guardian Email: ";
        const string LastUpdatedLabel = "Last updated: ";

        readonly static Regex NameRegex = new Regex(NoteParser.GetLabelExpression(NameLabel), RegexOptions.Compiled);
        readonly static Regex PhoneRegex = new Regex(NoteParser.GetLabelExpression(PhoneLabel), RegexOptions.Compiled);
        readonly static Regex EmailRegex = new Regex(NoteParser.GetLabelExpression(EmailLabel), RegexOptions.Compiled);
        readonly static Regex LastUpdatedRegex = new Regex(NoteParser.GetLabelExpression(LastUpdatedLabel), RegexOptions.Compiled);

        public static Guardian FromNotes(string notes)
        {
            var values = NoteParser.FromNotes(notes, NameRegex, PhoneRegex, EmailRegex, LastUpdatedRegex).ToArray();
            return new Guardian(values[0], values[1], values[2], ParseDateTime(values[3]));
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

        public override string ToString()
        {
            return NoteParser.ToNotes(
                (NameLabel, Name), 
                (PhoneLabel, PhoneNumber),
                (EmailLabel, Email), 
                (LastUpdatedLabel, LastUpdated?.ToLongDateString()));
        }
    }
}
