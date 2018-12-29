namespace YouthCenterSignIn.Logic.Data
{
    public class Guardian : NotifyBase
    {
        public Guardian()
        { }

        public Guardian(string name, string phoneNumber, string email = null)
        {
            Name = name;
            PhoneNumber = phoneNumber;
            Email = email;
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

        public override string ToString() => $"Guardian's Name: {Name}\r\nGuardian's Phone: {PhoneNumber}\r\nGuardian Email: {Email}";
    }
}
