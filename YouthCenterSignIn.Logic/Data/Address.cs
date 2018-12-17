using System.Linq;

namespace YouthCenterSignIn.Logic.Data
{
    public class Address : NotifyBase
    {
        public Address(string address1, string city, string state)
        {
            StreetAddress = address1;
            City = city;
            State = state;
        }

        string streetAddress;
        public string StreetAddress
        {
            get => streetAddress;
            set { streetAddress = value; OnPropertyChanged(); }
        }

        string city;
        public string City
        {
            get => city;
            set { city = value; OnPropertyChanged(); }
        }

        string state;
        public string State
        {
            get => state;
            set { state = value; OnPropertyChanged(); }
        }

        public bool IsValid()
        {
            string[] info = { StreetAddress, City, State };
            return info.All(i => !string.IsNullOrWhiteSpace(i));
        }
    }
}