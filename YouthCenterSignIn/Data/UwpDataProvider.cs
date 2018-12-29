
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Windows.ApplicationModel.Contacts;
using Windows.ApplicationModel.Core;
using Windows.Storage;
using Windows.UI.Core;
using Windows.UI.Popups;
using YouthCenterSignIn.Logic.Data;

namespace YouthCenterSignIn
{
    public class UwpDataProvider : DataProvider
    {
        internal Graph Graph { get; } = new Graph();

        public override async Task ShowMessage(string message = null, Exception exception = null)
        {
            if (!string.IsNullOrWhiteSpace(message))
            {
                await CoreApplication.MainView.CoreWindow.Dispatcher.RunAsync(CoreDispatcherPriority.Normal, async () =>
                {
                    await new MessageDialog(message).ShowAsync();
                });
            }

            if (exception != null)
            {
                //TODO add reporting
            }
        }

        #region People

        public override async Task<List<Person>> GetPeople()
        {
            if (!Graph.IsAuthenticated)
            {
                await ShowAuthenticationMessage();
                return new List<Person>();
            }

            var contactStore = await ContactManager.RequestStoreAsync(ContactStoreAccessType.AllContactsReadWrite);

            return (await (await GetContactList(contactStore)).GetContactReader().ReadBatchAsync()).Contacts
                .Select(c => ContactToPerson(c))
                .Where(c => c != null)
                .ToList();
        }

        internal async Task ShowAuthenticationMessage()
        {
            await ShowMessage("Please login to a Microsoft account on the logs page.");
        }

        Person ContactToPerson(Contact contact)
        {
            try
            {
                string id = contact.Id;
                string firstName = contact.FirstName;
                string lastName = contact.LastName;
                DateTimeOffset date = contact.ImportantDates.FirstOrDefault(d => d.Kind == ContactDateKind.Birthday).ToDateTimeOffset();
                Address address = contact.Addresses.FirstOrDefault().ToFullAddress();

                return new Person(id, firstName, lastName, date, address);
            }
            catch (Exception ex)
            {
                var reportTask = ShowMessage(exception: ex);
                return null;
            }
        }

        protected override async Task<string> AddPersonToData(Person person)
        {
            try
            {
                var contact = PersonToContact(person);
                await SaveContact(contact);
                return contact.Id;
            }
            catch (Exception ex)
            {
                await ShowMessage("Oops! Something went wrong while signing you up. Sorry about that...", ex);
                return null;
            }
        }

        Contact PersonToContact(Person person)
        {
            var contact = new Contact
            {
                FirstName = person.FirstName,
                LastName = person.LastName,
                Notes = person.Guardian?.ToString()
            };

            if (!string.IsNullOrWhiteSpace(person.Guardian?.PhoneNumber))
                contact.Phones.Add(new ContactPhone() { Kind = ContactPhoneKind.Mobile, Number = person.Guardian?.PhoneNumber });
            contact.Addresses.Add(person.Address.ToContactAddress());
            contact.ImportantDates.Add(person.BirthDate.ToContactDate());

            return contact;
        }

        async Task SaveContact(Contact contact)
        {
            var contactStore = await ContactManager.RequestStoreAsync(ContactStoreAccessType.AppContactsReadWrite);
            ContactList contactList = await GetContactList(contactStore);
            await contactList.SaveContactAsync(contact);
        }

        async Task<ContactList> GetContactList(ContactStore contactStore)
        {
            const string contactListName = "Youth Center";

            var contactLists = await contactStore.FindContactListsAsync();
            var contactList = contactLists.FirstOrDefault(l => l.DisplayName == contactListName);
            if (contactList == null)
            {
                contactList = await contactStore.CreateContactListAsync(contactListName);
                await contactList.SaveAsync();
            }

            return contactList;
        }

        #endregion

        #region Json

        protected override async Task<string> GetJsonFileContent(string fileName)
        {
            var rootFolder = await GetRootFolder();
            if (rootFolder == null)
                return null;

            var item = await rootFolder.TryGetItemAsync(fileName);
            if (item is IStorageFile file)
                return await FileIO.ReadTextAsync(file);
            else
                return null;
        }

        private async Task<StorageFolder> GetRootFolder()
        {
            try
            {
                var userPath = UserDataPaths.GetForUser(App.User).Profile;
                var userFolder = await StorageFolder.GetFolderFromPathAsync(userPath);
                var oneDriveFolder = await userFolder.GetFolderAsync("OneDrive");
                return await oneDriveFolder.CreateFolderAsync("Youth Center Sign In", CreationCollisionOption.OpenIfExists);
            }
            catch (UnauthorizedAccessException)
            {
                await ShowMessage("Could not access the folder, please give permission to the app.");
                await Windows.System.Launcher.LaunchUriAsync(new Uri("ms-settings:privacy-broadfilesystemaccess"));
                return null;
            }
        }

        protected override string GetJsonSetting(string key)
        {
            return (string)ApplicationData.Current.LocalSettings.Values[key];
        }

        protected override async Task SetJsonFileContent(string fileName, string json)
        {
            var rootFolder = await GetRootFolder();
            if (rootFolder == null)
                return;

            var file = await rootFolder.CreateFileAsync(fileName, CreationCollisionOption.OpenIfExists);
            await FileIO.WriteTextAsync(file, json);
        }

        protected override void SetJsonSetting(string key, string json)
        {
            ApplicationData.Current.LocalSettings.Values[key] = json;
        }

        #endregion
    }

    static class ContactExtensions
    {
        #region Dates

        public static DateTimeOffset ToDateTimeOffset(this ContactDate contactDate)
        {
            if (contactDate == null || contactDate.Year == null || contactDate.Month == null || contactDate.Day == null)
                return default(DateTimeOffset);

            return new DateTimeOffset(new DateTime(contactDate.Year.Value, Convert.ToInt32(contactDate.Month.Value), Convert.ToInt32(contactDate.Day.Value)));
        }

        public static ContactDate ToContactDate(this DateTimeOffset dateTimeOffset)
        {
            return new ContactDate()
            {
                Year = dateTimeOffset.Year,
                Month = Convert.ToUInt32(dateTimeOffset.Month),
                Day = Convert.ToUInt32(dateTimeOffset.Day)
            };
        }

        #endregion

        #region Addresses

        public static Address ToFullAddress(this ContactAddress address)
        {
            if (address == null)
                return null;

            return new Address(address.StreetAddress, city: address.Locality, state: address.Region);
        }

        public static ContactAddress ToContactAddress(this Address address)
        {
            return new ContactAddress()
            {
                StreetAddress = address.StreetAddress,
                Locality = address.City,
                Region = address.State
            };
        }

        #endregion
    }
}
