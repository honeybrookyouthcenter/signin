using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn.Logic
{
    public class PersonSearch : ReadOnlyObservableCollection<Person>, INotifyPropertyChanged
    {
        public PersonSearch() : base(new ObservableCollection<Person>())
        { }

        string searchText;
        public string SearchText
        {
            get => searchText;
            set
            {
                searchText = value;

                UpdateSearchResults();

                OnPropertyChanged();
            }
        }

        private async void UpdateSearchResults()
        {
            var search = SearchText.ToLower();
            if (string.IsNullOrWhiteSpace(search))
            {
                Items.Clear();
                return;
            }

            var results = (await Person.GetPeople()).Where(p => p.FullName.ToLower().Contains(search));
            results.OrderByDescending(p => p.FullName.ToLower().StartsWith(search));

            foreach (var person in Items.ToArray())
            {
                if (!results.Contains(person))
                    Items.Remove(person);
            }

            foreach (var person in results)
            {
                if (!Items.Contains(person))
                    Items.Add(person);
            }
        }

        protected void OnPropertyChanged([System.Runtime.CompilerServices.CallerMemberName] string propertyName = "")
        {
            OnPropertyChanged(new PropertyChangedEventArgs(propertyName));
        }
    }
}