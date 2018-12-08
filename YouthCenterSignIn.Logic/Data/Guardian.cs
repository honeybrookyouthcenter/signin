using System.ComponentModel;

namespace YouthCenterSignIn.Logic.Data
{
    public class Guardian : INotifyPropertyChanged
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

        public bool HasValidInfo(out string issues)
        {
            issues = "";

            return true; //TOOD
        }

        public static Guardian FromText(string notes)
        {
            //TODO
            string name = "";
            string phoneNumber = "";
            string email = "";

            return new Guardian(name, phoneNumber, email);
        }

        public override string ToString() => $"Guardian's Name: {Name}\r\nGuardian's Phone: {PhoneNumber}\r\nGuardian Email: {Email}";

        #region Notify

        /// <summary>
        /// Property Changed event
        /// </summary>
        public event PropertyChangedEventHandler PropertyChanged;

        /// <summary>
        /// Fire the PropertyChanged event
        /// </summary>
        /// <param name="propertyName">Name of the property that changed (defaults from CallerMemberName)</param>
        protected void OnPropertyChanged([System.Runtime.CompilerServices.CallerMemberName] string propertyName = "")
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        #endregion
    }
}
